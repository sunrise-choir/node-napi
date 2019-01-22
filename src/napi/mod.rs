#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use napi_sys::*;
use std::collections::BTreeMap;
use std::debug_assert;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;

use failure::Error;

#[derive(Debug, Fail)]
enum NapiError {
    #[fail(display = "unable to create string")]
    UnableToCreateString,
}

pub fn wrap_unsafe_create<T>(
    env: napi_env,
    item: T,
    f: unsafe extern "C" fn(napi_env, T, *mut napi_value) -> napi_status,
) -> napi_value {
    let mut result: napi_value = ptr::null_mut();
    let status = unsafe { f(env, item, &mut result) };
    debug_assert!(status == napi_status_napi_ok);
    result
}

pub fn wrap_unsafe_get<T: Default>(
    env: napi_env,
    value: napi_value,
    f: unsafe extern "C" fn(napi_env, napi_value, *mut T) -> napi_status,
) -> T {
    let mut result: T = T::default();
    let status = unsafe { f(env, value, &mut result) };
    debug_assert!(status == napi_status_napi_ok);
    result
}

pub fn throw_error(env: napi_env, err: Error) {
    let msg = CString::new(err.to_string()).unwrap();
    let status = unsafe { napi_throw_error(env, ptr::null(), msg.as_ptr() as *const c_char) };

    debug_assert!(status == napi_status_napi_ok)
}

pub fn create_error(env: napi_env, err: Error) -> napi_value {
    let mut result: napi_value = ptr::null_mut();
    let msg = create_string_utf8(env, &err.to_string());

    //TODO
    let status = unsafe { napi_create_error(env, ptr::null_mut(), msg, &mut result) };
    debug_assert!(status == napi_status_napi_ok);

    result
}

pub fn create_object(env: napi_env) -> napi_value {
    let mut object: napi_value = ptr::null_mut();

    let status = unsafe { napi_create_object(env, &mut object) };
    debug_assert!(status == napi_status_napi_ok);

    object
}

pub fn get_undefined_value(env: napi_env) -> napi_value {
    let mut undefined_value: napi_value = ptr::null_mut();
    let status = unsafe { napi_get_undefined(env, &mut undefined_value) };
    debug_assert!(status == napi_status_napi_ok);

    undefined_value
}

pub fn get_null_value(env: napi_env) -> napi_value {
    let mut null_value: napi_value = ptr::null_mut();
    let status = unsafe { napi_get_null(env, &mut null_value) };
    debug_assert!(status == napi_status_napi_ok);

    null_value
}

pub fn get_this(env: napi_env, info: napi_callback_info) -> napi_value {
    let mut num_args = 0;
    let mut args: Vec<napi_value> = Vec::with_capacity(num_args);
    let mut this = ptr::null_mut();

    let status = unsafe {
        napi_get_cb_info(
            env,
            info,
            &mut num_args,
            args.as_mut_ptr(),
            &mut this,
            ptr::null_mut(),
        )
    };

    debug_assert!(status == napi_status_napi_ok);
    this

}
pub fn get_arg(env: napi_env, info: napi_callback_info, arg_index: usize) -> napi_value {
    let mut num_args = arg_index + 1;
    let mut args: Vec<napi_value> = Vec::with_capacity(num_args);

    let status = unsafe {
        napi_get_cb_info(
            env,
            info,
            &mut num_args,
            args.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };

    debug_assert!(status == napi_status_napi_ok);

    unsafe { args.set_len(num_args) }

    *args.get(arg_index).unwrap_or(&get_undefined_value(env))
}

pub fn check_is_buffer(env: napi_env, value: napi_value) -> bool {
    let mut result = false;
    let status = unsafe { napi_is_buffer(env, value, &mut result) };
    debug_assert!(status == napi_status_napi_ok);

    result
}

pub fn get_buffer_info(env: napi_env, buffer: napi_value) -> (*mut u8, usize) {
    let mut buff_size = 0;
    let mut p_buff: *mut c_void = ptr::null_mut();

    let status = unsafe { napi_get_buffer_info(env, buffer, &mut p_buff, &mut buff_size) };
    debug_assert!(status == napi_status_napi_ok);

    (p_buff as *mut u8, buff_size)
}

pub fn create_buffer_copy(env: napi_env, slice: &[u8]) -> napi_value {
    let mut _p_buff: *mut c_void = ptr::null_mut();
    let mut buffer: napi_value = ptr::null_mut();

    let status = unsafe {
        napi_create_buffer_copy(
            env,
            slice.len(),
            slice.as_ptr() as *const c_void,
            &mut _p_buff,
            &mut buffer,
        )
    };

    debug_assert!(status == napi_status_napi_ok);

    buffer
}

pub fn create_array_with_length(env: napi_env, length: usize) -> napi_value {
    let mut array: napi_value = ptr::null_mut();

    let status = unsafe { napi_create_array_with_length(env, length, &mut array) };
    debug_assert!(status == napi_status_napi_ok);

    array
}

pub fn create_string_utf8(env: napi_env, string: &str) -> napi_value {
    let mut result: napi_value = ptr::null_mut();
    let p_str: *const c_char = string.as_ptr() as *const c_char;

    let status = unsafe { napi_create_string_utf8(env, p_str, string.len(), &mut result) };
    debug_assert!(status == napi_status_napi_ok);

    result
}

pub fn get_string(env: napi_env, value: napi_value) -> Result<String, Error> {
    let mut string_length_value = ptr::null_mut();

    let length_value = create_string_utf8(env, &"length");

    let status = unsafe { napi_get_property(env, value, length_value, &mut string_length_value) };

    if status != napi_status_napi_ok {
        bail!(NapiError::UnableToCreateString)
    }

    let mut num: u32 = 0;
    let status = unsafe { napi_get_value_uint32(env, string_length_value, &mut num) };
    debug_assert!(status == napi_status_napi_ok);

    num += 1; //allow for null terminating c string

    let mut vec: Vec<u8> = Vec::with_capacity(num as usize);
    let cstr = unsafe { CStr::from_ptr(vec.as_ptr() as *const c_char) };
    let mut length = 0;

    let status = unsafe {
        napi_get_value_string_utf8(
            env,
            value,
            cstr.as_ptr() as *mut c_char,
            num as usize,
            &mut length,
        )
    };
    if status == napi_status_napi_string_expected {
        bail!(NapiError::UnableToCreateString)
    }
    debug_assert!(status == napi_status_napi_ok);

    unsafe { vec.set_len(length) }

    String::from_utf8(vec).or(Err(NapiError::UnableToCreateString.into()))
}

pub fn create_buffer(env: napi_env, len: usize) -> napi_value {
    let mut _p_buff: *mut c_void = ptr::null_mut();
    let mut buffer: napi_value = ptr::null_mut();

    let status = unsafe { napi_create_buffer(env, len, &mut _p_buff, &mut buffer) };
    debug_assert!(status == napi_status_napi_ok);

    buffer
}

pub fn create_reference(env: napi_env, value: napi_value) -> napi_ref {
    let mut reference: napi_ref = ptr::null_mut();

    let status = unsafe { napi_create_reference(env, value, 1, &mut reference) };
    debug_assert!(status == napi_status_napi_ok);

    reference
}

pub fn get_reference_value(env: napi_env, reference: napi_ref) -> napi_value {
    let mut value: napi_value = ptr::null_mut();

    let status = unsafe { napi_get_reference_value(env, reference, &mut value) };
    debug_assert!(status == napi_status_napi_ok);

    value
}

pub fn delete_reference(env: napi_env, reference: napi_ref) {
    let status = unsafe { napi_delete_reference(env, reference) };
    debug_assert!(status == napi_status_napi_ok)
}

pub fn create_int32(env: napi_env, num: i32) -> napi_value {
    wrap_unsafe_create(env, num, napi_create_int32)
}

pub struct NapiEnv {
    pub env: napi_env,
}

pub fn get_typeof(env: napi_env, value: napi_value) -> napi_valuetype {
    let mut result = 0;
    let status = unsafe { napi_typeof(env, value, &mut result) };
    debug_assert!(status == napi_status_napi_ok);

    result
}

pub struct NapiArray {
    pub env: napi_env,
    pub array: napi_value,
    pub current_index: u32,
    pub length: u32,
}

impl NapiArray {
    pub fn from_existing(env: napi_env, array: napi_value) -> NapiArray {
        let mut length = 0;
        let status = unsafe { napi_get_array_length(env, array, &mut length) };
        debug_assert!(status == napi_status_napi_ok);

        NapiArray {
            env,
            array,
            length,
            current_index: 0,
        }
    }
    pub fn with_capacity(env: napi_env, capacity: usize) -> NapiArray {
        let array = create_array_with_length(env, capacity);
        NapiArray {
            env,
            array,
            length: 0,
            current_index: 0,
        }
    }

    pub fn push(&mut self, elem: napi_value) {
        //TODO: the push function (in push_array) could be stored in this object instead of having to get it for
        //every call to push_array.
        push_array(self.env, self.array, elem)
    }
}

impl Iterator for NapiArray {
    type Item = napi_value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.length {
            return None;
        }

        let mut value: napi_value = ptr::null_mut();
        let status =
            unsafe { napi_get_element(self.env, self.array, self.current_index, &mut value) };
        debug_assert!(status == napi_status_napi_ok);

        self.current_index += 1;

        Some(value)
    }
}

impl ExactSizeIterator for NapiArray {
    fn len(&self) -> usize {
        self.length as usize
    }
}

pub fn get_object_map(env: napi_env, object: napi_value) -> BTreeMap<String, napi_value> {
    //get keys of object.
    let mut map = BTreeMap::<String, napi_value>::new();
    let mut keys_value = ptr::null_mut();
    let status = unsafe { napi_get_property_names(env, object, &mut keys_value) };
    debug_assert!(status == napi_status_napi_ok);

    for key in NapiArray::from_existing(env, keys_value) {
        let mut value: napi_value = ptr::null_mut();
        let status = unsafe { napi_get_property(env, object, key, &mut value) };
        debug_assert!(status == napi_status_napi_ok);

        if let Ok(key_string) = get_string(env, key) {
            map.insert(key_string, value);
        }
    }

    map
}

pub fn push_array(env: napi_env, array: napi_value, elem: napi_value) {
    let mut return_value: napi_value = ptr::null_mut();
    let mut push_fn: napi_value = ptr::null_mut();
    let args: [napi_value; 1] = [elem];

    let status = unsafe {
        napi_get_named_property(env, array, "slice".as_ptr() as *const c_char, &mut push_fn)
    };
    debug_assert!(status == napi_status_napi_ok);

    let status = unsafe {
        napi_call_function(
            env,
            array,
            push_fn,
            1,
            &args[0] as *const napi_value,
            &mut return_value,
        )
    };

    debug_assert!(status == napi_status_napi_ok);
}

pub fn slice_buffer(env: napi_env, buff: napi_value, beginning: usize, end: usize) -> napi_value {
    let mut slice_fn: napi_value = ptr::null_mut();
    let mut args: [napi_value; 2] = [ptr::null_mut(), ptr::null_mut()];
    let mut return_value: napi_value = ptr::null_mut();

    args[0] = create_int32(env, beginning as i32);
    args[1] = create_int32(env, end as i32);

    let status = unsafe {
        napi_get_named_property(env, buff, "slice".as_ptr() as *const c_char, &mut slice_fn)
    };
    debug_assert!(status == napi_status_napi_ok);

    let status = unsafe {
        napi_call_function(
            env,
            buff,
            slice_fn,
            2,
            &args[0] as *const napi_value,
            &mut return_value,
        )
    };
    debug_assert!(status == napi_status_napi_ok);

    return_value
}

pub fn define_class(
    env: napi_env,
    name: &str,
    constructor: napi_callback,
    data: *mut c_void,
    properties: &[napi_property_descriptor],
) -> napi_value {
    let mut result: napi_value = ptr::null_mut();

    let status = unsafe {
        napi_define_class(
            env,
            name.as_ptr() as *const ::std::os::raw::c_char,
            name.len(),
            constructor,
            data,
            properties.len(),
            properties.as_ptr(),
            &mut result,
        )
    };

    debug_assert!(status == napi_status_napi_ok);

    result
}
