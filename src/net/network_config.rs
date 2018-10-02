use std::default::Default;

#[derive(Clone)]
pub struct NetworkConfig
{
    pub max_packet_size: usize,
    pub max_fragments: u8,
    pub fragment_size: u16,
    pub fragment_reassembly_buffer_size: usize,
    pub receive_buffer_max_size: u16,
}

impl Default for NetworkConfig{
    fn default() -> Self {
        Self {
            max_packet_size: 16 * 1024,
            max_fragments: 16,
            fragment_size: 1024,
            fragment_reassembly_buffer_size: 64,
            receive_buffer_max_size: 1500,
        }
    }
}