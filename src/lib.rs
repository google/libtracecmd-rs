// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[allow(
    clippy::upper_case_acronyms,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code
)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to open .dat file")]
    Open,
    #[error("failed to get tep_handle")]
    Handle,
    #[error("failed to find tep_event")]
    FindEvent,
    #[error("failed to find tep_field")]
    FindField,
    #[error("invalid pid: {0}")]
    InvalidPid(String),
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("invalid string: {0}")]
    InvalidString(std::str::Utf8Error),
    #[error("failed to read a field")]
    ReadField,
}

type Result<T> = std::result::Result<T, Error>;

unsafe fn cptr_to_string(ptr: *mut i8) -> Result<String> {
    let c_str: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(ptr) };
    Ok(c_str.to_str().map_err(Error::InvalidString)?.to_string())
}

pub struct Input(*mut bindings::tracecmd_input);

impl Input {
    pub fn new(path: &str) -> Result<Self> {
        // TODO: Support open flags.
        // TODO: safe reason
        let handle = unsafe { bindings::tracecmd_open(path.as_ptr() as *mut i8, 0) };
        if handle.is_null() {
            return Err(Error::Open);
        }

        Ok(Input(handle))
    }

    pub fn handle(&self) -> Result<Handle> {
        let ret = unsafe { bindings::tracecmd_get_tep(self.0) };
        if ret.is_null() {
            Err(Error::Handle)
        } else {
            Ok(Handle(ret))
        }
    }

    pub fn find_event(&self, rec: &Record) -> Result<Event> {
        let handle = self.handle()?;
        let ptr = unsafe { bindings::tep_find_event_by_record(handle.0, rec.0) };
        if ptr.is_null() {
            return Err(Error::FindEvent);
        }
        let name = unsafe { cptr_to_string((*ptr).name) }.expect("string");

        Ok(Event { ptr, name })
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        // Safe because `self.0` must be a valid pointer.
        unsafe {
            bindings::tracecmd_close(self.0);
        }
    }
}

// TODO: free pointer in drop()
pub struct Handle(*mut bindings::tep_handle);

impl Handle {
    pub fn pid(&self, rec: &Record) -> i32 {
        unsafe { bindings::tep_data_pid(self.0, rec.0) }
    }
}

// TODO: free pointer in drop()
pub struct Record(*mut bindings::tep_record);

impl Record {
    pub fn ts(&self) -> u64 {
        unsafe { *self.0 }.ts
    }
}

pub struct Event {
    ptr: *mut bindings::tep_event,
    pub name: String,
}

pub struct Field(*mut bindings::tep_format_field);
impl Record {
    //tep_read_number(tep, record->data + field->offset,
    //field->size);
    pub fn read_field(&self, field: &Field) -> Result<u64> {
        let mut val = 0;
        let ret = unsafe { bindings::tep_read_number_field(field.0, (*self.0).data, &mut val) };
        if ret == 0 {
            return Err(Error::ReadField);
        }
        Ok(val)
    }
}

impl Event {
    pub fn print_field(&self, rec: &Record) {
        let mut seq: bindings::trace_seq = Default::default();
        unsafe {
            bindings::trace_seq_init(&mut seq);
            bindings::trace_seq_reset(&mut seq);

            bindings::tep_record_print_fields(&mut seq, rec.0, self.ptr);
            bindings::trace_seq_terminate(&mut seq);
        };
        let msg = unsafe { std::slice::from_raw_parts(seq.buffer as *mut u8, seq.len as usize) };
        println!("fields: {:?}", std::str::from_utf8(msg).unwrap());
    }

    pub fn get_common_field_val(&self, rec: &Record, name: &str) -> Result<u64> {
        let name = name.to_string() + "\0";
        let mut val = 0u64;
        let mut seq: bindings::trace_seq = Default::default();
        let err = unsafe {
            bindings::trace_seq_init(&mut seq);
            bindings::trace_seq_reset(&mut seq);

            let err = bindings::tep_get_common_field_val(
                &mut seq,
                self.ptr,
                name.as_ptr() as *const i8,
                rec.0,
                &mut val as *mut u64,
                1,
            );

            bindings::trace_seq_terminate(&mut seq);
            err
        };

        if err != 0 {
            let msg =
                unsafe { std::slice::from_raw_parts(seq.buffer as *mut u8, seq.len as usize) };
            println!("Error: {:?}", std::str::from_utf8(msg).unwrap());
            return Err(Error::FindField);
        }
        Ok(val)
    }

    pub fn find_field(&self, rec: &Record, name: &str) -> Result<u64> {
        let name = name.to_string() + "\0";
        let mut val = 0u64;
        let mut seq: bindings::trace_seq = Default::default();
        let err = unsafe {
            bindings::trace_seq_init(&mut seq);
            bindings::trace_seq_reset(&mut seq);

            let err = bindings::tep_get_field_val(
                &mut seq,
                self.ptr,
                name.as_ptr() as *const i8,
                rec.0,
                &mut val as *mut u64,
                1,
            );

            bindings::trace_seq_terminate(&mut seq);
            err
        };

        if err != 0 {
            let msg =
                unsafe { std::slice::from_raw_parts(seq.buffer as *mut u8, seq.len as usize) };
            println!("Error: {:?}", std::str::from_utf8(msg).unwrap());

            return Err(Error::FindField);
        }
        Ok(val)
    }
}

pub trait Handler {
    type DataType: Default;

    fn callback(input: &mut Input, rec: &mut Record, cpu: i32, data: &mut Self::DataType) -> i32;

    unsafe extern "C" fn c_callback(
        input: *mut bindings::tracecmd_input,
        rec: *mut bindings::tep_record,
        cpu: i32,
        raw_data: *mut std::ffi::c_void,
    ) -> i32 {
        let mut input = Input(input);
        let mut rec = Record(rec);

        // TODO: Remove this unnecessary data copy?
        // What I only need here is a type conversion.
        let mut data: Self::DataType = Default::default();
        std::ptr::copy_nonoverlapping(
            raw_data,
            &mut data as *mut _ as *mut std::ffi::c_void,
            std::mem::size_of::<Self::DataType>(),
        );
        let res = Self::callback(&mut input, &mut rec, cpu, &mut data);
        std::ptr::copy_nonoverlapping(
            &mut data as *mut _ as *mut std::ffi::c_void,
            raw_data,
            std::mem::size_of::<Self::DataType>(),
        );

        std::mem::forget(input);
        std::mem::forget(rec);
        std::mem::forget(data); // TODO: remove `data` above or use `Pin`?

        res
    }

    fn process(input: &mut Input) -> std::result::Result<Self::DataType, i32> {
        let mut data: Self::DataType = Default::default();

        let ret = unsafe {
            bindings::tracecmd_iterate_events(
                input.0,
                // If `cpus` is null, `cpus` and `cpu_size` are ignored and all of CPUs will be
                // checked.
                std::ptr::null_mut(), /* cpus */
                0,                    /* cpu_size */
                Some(Self::c_callback),
                &mut data as *mut _ as *mut std::ffi::c_void,
            )
        };
        if ret == 0 {
            Ok(data)
        } else {
            Err(ret)
        }
    }

    fn process_multi(inputs: &mut [Input]) -> std::result::Result<Self::DataType, i32> {
        let mut data: Self::DataType = Default::default();
        let nr_handles = inputs.len() as i32;

        let mut handles = inputs.iter().map(|input| input.0).collect::<Vec<_>>();

        let ret = unsafe {
            bindings::tracecmd_iterate_events_multi(
                handles.as_mut_ptr(),
                nr_handles,
                Some(Self::c_callback),
                &mut data as *mut _ as *mut std::ffi::c_void,
            )
        };
        if ret == 0 {
            Ok(data)
        } else {
            Err(ret)
        }
    }
}
