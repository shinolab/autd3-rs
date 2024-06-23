use crate::local::soem_bindings;

use std::{ffi::CStr, fmt, slice};

use derive_more::{Deref, Display};

#[derive(Clone, Display)]
#[display(fmt = "{}, {}", name, desc)]
pub struct EthernetAdapter {
    desc: String,
    name: String,
}

impl EthernetAdapter {
    pub fn desc(&self) -> &str {
        &self.desc
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Deref)]
pub struct EthernetAdapters {
    #[deref]
    adapters: Vec<EthernetAdapter>,
}

impl EthernetAdapters {
    pub fn new() -> Self {
        let mut adapters = Vec::new();
        unsafe {
            let mut adapter = soem_bindings::ec_find_adapters();
            while !adapter.is_null() {
                if let Ok(name) = CStr::from_ptr(((*adapter).name).as_ptr()).to_str() {
                    adapters.push(EthernetAdapter {
                        desc: CStr::from_ptr(((*adapter).desc).as_ptr())
                            .to_str()
                            .unwrap_or("")
                            .to_string(),
                        name: name.to_string(),
                    });
                }
                adapter = (*adapter).next;
            }
            soem_bindings::ec_free_adapters(adapter);
            EthernetAdapters { adapters }
        }
    }
}

impl Default for EthernetAdapters {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a EthernetAdapters {
    type Item = &'a EthernetAdapter;
    type IntoIter = slice::Iter<'a, EthernetAdapter>;

    fn into_iter(self) -> slice::Iter<'a, EthernetAdapter> {
        self.adapters.iter()
    }
}
