#[macro_use]
extern crate rocket;
use base64::{engine::general_purpose, Engine as _};
use rocket::serde::{json::Json, Deserialize, Serialize};
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct SignatureResponse {
    signature: String,
    public_key: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct VerifyRequest<'a> {
    data: &'a str,
    signature: &'a str,
    public_key: &'a str,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct SignRequest<'a> {
    data: &'a str,
}

#[post("/sign", data = "<sign_request>")]
fn sign(sign_request: Json<SignRequest<'_>>) -> Json<SignatureResponse> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();

    let module =
        Module::from_file(&engine, "dilithium_code_sign_module.wasm.multivalue.wasm").unwrap();

    let mut linker = Linker::new(&engine);

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();

    let mut store = Store::new(&engine, wasi);

    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

    let instance = linker.instantiate(&mut store, &module).unwrap();

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or(anyhow::format_err!("failed to find `memory` export"))
        .unwrap();

    let alloc_fn = instance
        .get_typed_func::<i32, i32>(&mut store, "alloc")
        .unwrap();
    let dealloc_fn = instance
        .get_typed_func::<(i32, i32), ()>(&mut store, "dealloc")
        .unwrap();
    let sign_data_fn = instance
        .get_typed_func::<(i32, i32), (i32, i32)>(&mut store, "sign_data")
        .unwrap();

    let data_ptr = alloc_fn
        .call(&mut store, sign_request.data.len() as i32)
        .unwrap();

    unsafe {
        let raw = memory.data_ptr(&mut store).offset(data_ptr as isize);
        raw.copy_from(
            sign_request.data.as_bytes().as_ptr(),
            sign_request.data.len(),
        );
    };

    let (sig_ptr, pk_ptr) = sign_data_fn
        .call(
            &mut store,
            (data_ptr as i32, sign_request.data.len() as i32),
        )
        .unwrap();

    let mut sig_data = [0u8; 3293];
    memory
        .read(&store, sig_ptr as usize, &mut sig_data)
        .unwrap();

    let mut pk_data = [0u8; 1952];
    memory.read(&store, pk_ptr as usize, &mut pk_data).unwrap();

    dealloc_fn
        .call(&mut store, (sig_ptr, sig_data.len() as i32))
        .unwrap();
    dealloc_fn
        .call(&mut store, (pk_ptr, pk_data.len() as i32))
        .unwrap();
    let sig_data_encoded = general_purpose::STANDARD.encode(&sig_data);
    let pk_data_encoded = general_purpose::STANDARD.encode(&pk_data);
    Json(SignatureResponse {
        signature: sig_data_encoded,
        public_key: pk_data_encoded,
    })
}

#[post("/verify", data = "<verify_request>")]
fn verify(verify_request: Json<VerifyRequest<'_>>) -> Json<bool> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();

    let module =
        Module::from_file(&engine, "dilithium_code_sign_module.wasm.multivalue.wasm").unwrap();

    let mut linker = Linker::new(&engine);

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();

    let mut store = Store::new(&engine, wasi);

    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

    let instance = linker.instantiate(&mut store, &module).unwrap();

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or(anyhow::format_err!("failed to find `memory` export"))
        .unwrap();

    let alloc_fn = instance
        .get_typed_func::<i32, i32>(&mut store, "alloc")
        .unwrap();

    let verify_data_fn = instance
        .get_typed_func::<(i32, i32, i32, i32, i32, i32), i32>(&mut store, "verify_data")
        .unwrap();

    //decode signature, public key
    let signature_decoded = general_purpose::STANDARD
        .decode(&verify_request.signature)
        .unwrap();
    let public_key_decoded = general_purpose::STANDARD
        .decode(&verify_request.public_key)
        .unwrap();
    let data_decoded = verify_request.data.as_bytes().to_vec();

    //copy signature, public key, data to wasm memory
    let signature_ptr = alloc_fn
        .call(&mut store, signature_decoded.len() as i32)
        .unwrap();
    let public_key_ptr = alloc_fn
        .call(&mut store, public_key_decoded.len() as i32)
        .unwrap();
    let data_ptr = alloc_fn
        .call(&mut store, data_decoded.len() as i32)
        .unwrap();

    unsafe {
        let raw = memory.data_ptr(&mut store).offset(signature_ptr as isize);
        raw.copy_from(signature_decoded.as_ptr(), signature_decoded.len());
    };

    unsafe {
        let raw = memory.data_ptr(&mut store).offset(public_key_ptr as isize);
        raw.copy_from(public_key_decoded.as_ptr(), public_key_decoded.len());
    };

    unsafe {
        let raw = memory.data_ptr(&mut store).offset(data_ptr as isize);
        raw.copy_from(data_decoded.as_ptr(), data_decoded.len());
    };

    let response = verify_data_fn
        .call(
            &mut store,
            (
                data_ptr,
                data_decoded.len() as i32,
                signature_ptr,
                signature_decoded.len() as i32,
                public_key_ptr,
                public_key_decoded.len() as i32,
            ),
        )
        .unwrap();

    if response != 0 {
        Json(true)
    } else {
        Json(false)
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![sign, verify])
}
