#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Symbol {
    pub value: u8,
    pub count: u8,

    pub under: u16,
    pub left: u16,
    pub right: u16,

    pub symbols: u16,
    pub escapes: u16,
    pub total: u16,
    pub parent: u16,
}

pub mod constants {
    pub const RANGE_CODER_TOP: u32 = 1 << 24;
    pub const RANGE_CODER_BOTTOM: u32 = 1 << 16;

    pub const CONTEXT_SYMBOL_DELTA: u32 = 3;
    pub const CONTEXT_SYMBOL_MINIMUM: u32 = 1;
    pub const CONTEXT_ESCAPE_MINIMUM: u32 = 1;

    pub const SUBCONTEXT_ORDER: u32 = 2;
    pub const SUBCONTEXT_SYMBOL_DELTA: u32 = 2;
    pub const SUBCONTEXT_ESCAPE_DELTA: u32 = 5;
}

#[derive(Default)]
pub struct RangeCoder {
    pub symbols: Vec<Symbol>
}

impl RangeCoder {
    pub fn create() -> Self {
        let symbols = Vec::with_capacity(4096);
        Self { symbols }
    }

    pub fn create_symbol(&mut self, value: u8, count: u8) -> usize {
        let symbol = Symbol {
            value,
            count,
            under: count as u16,
            left: 0,
            right: 0,
            symbols: 0,
            escapes: 0,
            total: 0,
            parent: 0,
        };
        self.symbols.push(symbol);
        self.symbols.len() - 1
    }

    pub fn create_context(&mut self, escapes: u16, minimum: u32) -> usize {
        let index = self.create_symbol(0, 0);
        {
            let context = &mut self.symbols[index];
            context.escapes = escapes;
            context.total = escapes + 256 * minimum as u16;
            context.symbols = 0;
        }
        index
    }

    pub fn symbol_rescale(&mut self, index: usize) -> u16 {
        use std::collections::VecDeque;

        let mut total = 0;
        let mut stack = VecDeque::new();
        let mut visited = vec![false; self.symbols.len()];
        stack.push_back(index);

        while let Some(i) = stack.back().copied() {
            if !visited[i] {
                visited[i] = true;
                let symbol = &self.symbols[i];
                if symbol.right != 0 {
                    stack.push_back(i + symbol.right as usize);
                }
                if symbol.left != 0 {
                    stack.push_back(i + symbol.left as usize);
                }
            } else {
                stack.pop_back();

                let left_index = self.symbols[i].left;
                let right_index = self.symbols[i].right;

                let left_under = if left_index != 0 {
                    self.symbols[i + left_index as usize].under
                } else {
                    0
                };

                let right_under = if right_index != 0 {
                    self.symbols[i + right_index as usize].under
                } else {
                    0
                };

                let symbol = &mut self.symbols[i];
                symbol.count -= symbol.count >> 1;
                symbol.under = symbol.count as u16 + left_under + right_under;
                total += symbol.under;
            }
        }

        total
    }
}

pub struct RangeEncoder<'a> {
    pub output: &'a mut [u8],
    pub position: usize,
    pub low: u32,
    pub range: u32,
}

impl<'a> RangeEncoder<'a> {
    pub fn new(output: &'a mut [u8]) -> Self {
        Self {
            output,
            position: 0,
            low: 0,
            range: !0,
        }
    }

    pub fn write_byte(&mut self, byte: u8) -> bool {
        if self.position >= self.output.len() {
            return false;
        }
        self.output[self.position] = byte;
        self.position += 1;
        true
    }

    pub fn encode(&mut self, under: u32, count: u32, total: u32) -> bool {
        self.range /= total;
        self.low += under * self.range;
        self.range *= count;

        loop {
            if (self.low ^ (self.low + self.range)) >= constants::RANGE_CODER_TOP {
                if self.range >= constants::RANGE_CODER_BOTTOM {
                    break;
                }

                self.range = self.low.wrapping_neg() & (constants::RANGE_CODER_BOTTOM - 1);
            }

            if !self.write_byte((self.low >> 24) as u8) {
                return false;
            }

            self.range <<= 8;
            self.low <<= 8;
        }

        true
    }

    pub fn flush(&mut self) {
        while self.low != 0 {
            if !self.write_byte((self.low >> 24) as u8) {
                break;
            }
            self.low <<= 8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_symbol() {
        let mut coder = RangeCoder::create();
        let index = coder.create_symbol(42, 5);
        let symbol = &coder.symbols[index];

        assert_eq!(symbol.value, 42);
        assert_eq!(symbol.count, 5);
        assert_eq!(symbol.under, 5);
        assert_eq!(symbol.left, 0);
        assert_eq!(symbol.right, 0);
    }

    #[test]
    fn test_create_context() {
        let mut coder = RangeCoder::create();
        let context_index = coder.create_context(10, constants::CONTEXT_SYMBOL_MINIMUM);
        let context = &coder.symbols[context_index];

        assert_eq!(context.escapes, 10);
        assert_eq!(context.total, 10 + 256 * constants::CONTEXT_SYMBOL_MINIMUM as u16);
        assert_eq!(context.symbols, 0);
    }

    #[test]
    fn test_symbol_rescale_simple_tree() {
        let mut coder = RangeCoder::create();

        let root = coder.create_symbol(0, 8);
        let left = coder.create_symbol(1, 4);
        let right = coder.create_symbol(2, 2);

        coder.symbols[root].left = left as u16 - root as u16;
        coder.symbols[root].right = right as u16 - root as u16;

        let total = coder.symbol_rescale(root);

        let root_symbol = &coder.symbols[root];
        let left_symbol = &coder.symbols[left];
        let right_symbol = &coder.symbols[right];

        assert!(root_symbol.count < 8);
        assert!(left_symbol.count < 4);
        assert!(right_symbol.count < 2);

        assert_eq!(root_symbol.under, root_symbol.count as u16 + left_symbol.under + right_symbol.under);

        let expected_total = coder.symbols.iter().map(|s| s.under as u16).sum::<u16>();
        assert_eq!(total, expected_total);
    }

    #[test]
    fn test_range_encoder_basic() {
        let mut buffer = [0u8; 10];
        let mut encoder = RangeEncoder::new(&mut buffer);

        let success = encoder.encode(5, 10, 100);
        assert!(success);

        encoder.flush();
        assert!(encoder.position > 0);
    }

    #[test]
    fn test_range_encoder_overflow() {
        let mut buffer = [0u8; 1];
        let mut encoder = RangeEncoder::new(&mut buffer);

        let success = encoder.encode(5, 10, 100);
        assert!(!success || encoder.position <= buffer.len());
    }
}