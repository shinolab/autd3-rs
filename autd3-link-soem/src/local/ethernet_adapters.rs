use std::fmt;

use crate::local::soem_bindings;

use std::ffi::CStr;
use std::ops::Index;
use std::slice;

/// Ethernet adapter
#[derive(Clone)]
pub struct EthernetAdapter {
    desc: String,
    name: String,
}

impl EthernetAdapter {
    /// Description of the adapter
    pub fn desc(&self) -> &str {
        &self.desc
    }

    /// Name of the adapter
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct EthernetAdapters {
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

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for EthernetAdapters {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for EthernetAdapters {
    type Output = EthernetAdapter;
    fn index(&self, index: usize) -> &Self::Output {
        &self.adapters[index]
    }
}

impl<'a> IntoIterator for &'a EthernetAdapters {
    type Item = &'a EthernetAdapter;
    type IntoIter = slice::Iter<'a, EthernetAdapter>;

    fn into_iter(self) -> slice::Iter<'a, EthernetAdapter> {
        self.adapters.iter()
    }
}

impl fmt::Display for EthernetAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.name, self.desc)
    }
}
