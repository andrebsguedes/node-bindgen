
use std::ptr;

use log::trace;

use crate::TryIntoJs;
use crate::sys::napi_value;
use crate::sys::napi_env;
use crate::val::JsEnv;
use crate::NjError;


/// pass rust byte arry as Node.js ArrayBuffer
pub struct ArrayBuffer{
    data: Vec<u8>
}

impl ArrayBuffer {

    pub fn new(data: Vec<u8>) -> Self {
        Self { data}
    }

    extern "C" fn finalize_buffer(_env: napi_env,_finalize_data: *mut ::std::os::raw::c_void,
        finalize_hint: *mut ::std::os::raw::c_void
    ) {

        trace!("finalize array buffer");
        unsafe {
            // use hint to reconstruct box instead of finalize data
            let ptr: *mut Vec<u8> = finalize_hint as *mut Vec<u8>;
            let _rust = Box::from_raw(ptr);
        }

    }

}

impl TryIntoJs for ArrayBuffer {

    fn try_to_js(self, js_env: &JsEnv) -> Result<napi_value,NjError> {

        let len = self.data.len();

        let box_data = Box::new(self.data);

        let mut napi_buffer = ptr::null_mut();

        // get pointer to vec's buffer
        let data_buffer = box_data.as_ptr();

        // get raw pointer to box, this will be used to reconstruct box
        let data_box_ptr = Box::into_raw(box_data) as *mut core::ffi::c_void;

        crate::napi_call_result!(
            crate::sys::napi_create_external_arraybuffer(
                js_env.inner(),
                data_buffer as *mut core::ffi::c_void ,
                len,
                Some(Self::finalize_buffer),
                data_box_ptr,
                &mut napi_buffer
            )
        )?;

        Ok(napi_buffer)

    }
}


// impl JSValue for ArrayBuffer {
//
//     fn convert_to_rust(env: &JsEnv, js_value: napi_value) -> Result<Self, NjError> {
//
//         if !env.is_buffer(js_value)? {
//             return Err(NjError::Other(format!("Type is not a buffer")))
//         }
//
//         use crate::sys::napi_get_buffer_info;
//
//         let mut data =  ptr::null_mut();
//         let mut chars: [u8; 1024] = [0;1024];
//         let mut size: size_t = 0;
//
//         napi_call_result!(
//             napi_get_buffer_info(env.inner(), js_value, chars.as_mut_ptr() as *mut i8,1024,&mut size)
//         )?;
//
//         let my_chars: Vec<u8> = chars[0..size].into();
//
//         String::from_utf8(my_chars).map_err(|err| err.into())
//     }
//
// }

use std::fmt;
use std::fmt::Debug;

impl Debug for ArrayBuffer {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("ArrayBuffer len: {}",self.data.len()))
    }

}



