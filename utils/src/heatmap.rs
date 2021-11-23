use std::convert::TryInto;

pub fn get_heatmap(input: &[u8; 3500]) -> &[u8; 2816] {
    (&input[26..26 + (44 * 64)]).try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::get_heatmap;

    #[test]
    fn it_works() {
        let data = include_bytes!("../../heatmap.bin").to_vec();
        let data: &[u8; 3500] = &data.try_into().unwrap();
        let result = get_heatmap(data);
        assert_eq!(*result.first().unwrap(), 0xB4);
        assert_eq!(*result.last().unwrap(), 0xB4);
        assert_eq!(result.len(), 44 * 64);
    }
}
