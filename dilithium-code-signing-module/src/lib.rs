use pqc_dilithium::*;
/// Allocate memory into the module's linear memory
/// and return the offset to the start of the block.
#[no_mangle]
pub fn alloc(len: usize) -> *mut u8 {
    // create a new mutable buffer with capacity `len`
    let mut buf = Vec::with_capacity(len);
    // take a mutable pointer to the buffer
    let ptr = buf.as_mut_ptr();
    // take ownership of the memory block and
    // ensure that its destructor is not
    // called when the object goes out of scope
    // at the end of the function
    std::mem::forget(buf);
    // return the pointer so the runtime
    // can write data at this offset
    return ptr;
}

#[no_mangle]
pub unsafe fn sign_data(data: *mut u8, data_len: usize) -> (*mut u8, *mut u8) {
    //Reading data from linear memory
    let data = Vec::from_raw_parts(data, data_len, data_len);
    //Generating Key Pair
    let keys = Keypair::generate();
    //Extracting pk
    let mut pk = keys.public.to_vec();
    //Signing the data
    let mut signature = keys.sign(&data).to_vec();
    //Converting signature and pk to raw pointers
    let sig_ptr = signature.as_mut_ptr();
    let pk_ptr = pk.as_mut_ptr();
    // take ownership of the memory block and
    // ensure that its destructor is not
    // called when the object goes out of scope
    // at the end of the function
    std::mem::forget(signature);
    std::mem::forget(pk);
    //Returning the pointers
    (sig_ptr, pk_ptr)
}

#[no_mangle]
pub unsafe fn verify_data(
    data: *mut u8,
    data_len: usize,
    signature: *mut u8,
    signature_len: usize,
    pk: *mut u8,
    pk_len: usize,
) -> i32 {
    //Reading data, signature and pk from linear memory
    let data = Vec::from_raw_parts(data, data_len, data_len);
    let signature = Vec::from_raw_parts(signature, signature_len, signature_len);
    let pk = Vec::from_raw_parts(pk, pk_len, pk_len);

    //Verifying the signature
    match verify(&signature, &data, &pk) {
        Ok(_) => return 1,
        Err(_) => return 0,
    }
}

//deallocator function for a memory block by calling std::mem::drop using the desired value,
//which effectively takes ownership of data and goes out of scope.
#[no_mangle]
pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    let data = Vec::from_raw_parts(ptr, size, size);

    std::mem::drop(data);
}
