#![feature(avx512_target_feature)]
#![feature(stdsimd)]

use std::arch::x86_64::*;

#[target_feature(enable = "avx,avx2,avx512f,avx512bw,avx512cd,avx512vbmi")]
unsafe fn process_heatmap_internal(input: &[u8; 2816], results: &mut [(u32, u32); 10]) -> u32 {
    let indexer = _mm512_set_epi16(
        31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9,
        8, 7, 6, 5, 4, 3, 2, 1, 0,
    );
    let add_1 = _mm512_set1_epi32(1);
    let add_half_row = _mm512_set1_epi16(32);
    let zero = _mm512_setzero_epi32();
    #[allow(unused_macros)]
    macro_rules! debug_avx512 {
        ($row: expr $(,)?) => {{
            let debug_charset = _mm512_set_epi16(
                70, 69, 68, 67, 66, 65, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 70, 69, 68, 67, 66,
                65, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48,
            );
            let byte_shift = _mm_set_epi64x(0, 8);
            let nibble_shift = _mm_set_epi64x(0, 4);
            let h = _mm512_cvtepi8_epi16(_mm512_extracti64x4_epi64::<1>(*$row));
            let l = _mm512_cvtepi8_epi16(_mm512_castsi512_si256(*$row));
            let out = [
                _mm512_or_si512(
                    _mm512_permutexvar_epi16(_mm512_srl_epi16(l, nibble_shift), debug_charset),
                    _mm512_sll_epi16(_mm512_permutexvar_epi16(l, debug_charset), byte_shift),
                ),
                _mm512_or_si512(
                    _mm512_permutexvar_epi16(_mm512_srl_epi16(h, nibble_shift), debug_charset),
                    _mm512_sll_epi16(_mm512_permutexvar_epi16(h, debug_charset), byte_shift),
                ),
            ];
            println!(
                "{}",
                std::str::from_utf8(std::slice::from_raw_parts(out.as_ptr() as *const u8, 128))
                    .unwrap(),
            );
        }};
    }

    let mut buf_a = [_mm512_setzero_epi32(); 88];
    let mut buf_b = [_mm512_setzero_epi32(); 88];
    let inverteds = {
        let mut inverteds = [_mm512_setzero_epi32(); 44];
        let inverted_floor = _mm512_set1_epi8(0xB0u8 as i8);
        let mut i = 44;
        let mut input = (input.as_ptr() as *const i8).offset(64 * 44);
        while i != 0 {
            i -= 1;
            input = input.offset(-64);
            let inverted = _mm512_subs_epu8(inverted_floor, _mm512_loadu_epi8(input));
            inverteds[i] = inverted;
        }
        inverteds
    };

    {
        let mut indexer = _mm512_add_epi16(_mm512_set1_epi16(64 * 44 + 1), indexer);
        let mut i = 44;
        while i != 0 {
            i -= 1;
            let row = inverteds[i];
            let mask = _mm512_test_epi8_mask(row, row);
            indexer = _mm512_sub_epi16(indexer, add_half_row);
            buf_a[i * 2 + 1] = _mm512_maskz_mov_epi16((mask >> 32) as u32, indexer);
            indexer = _mm512_sub_epi16(indexer, add_half_row);
            buf_a[i * 2 + 0] = _mm512_maskz_mov_epi16((mask >> 0) as u32, indexer);
        }
    }

    let (indices, result) = {
        let shuffle_right = _mm512_set_epi16(
            32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11,
            10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
        );
        let shuffle_left = _mm512_set_epi16(
            30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9,
            8, 7, 6, 5, 4, 3, 2, 1, 0, 63,
        );
        let shift_right = _mm512_set_epi16(
            31, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11,
            10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
        );
        let shift_left = _mm512_set_epi16(
            30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9,
            8, 7, 6, 5, 4, 3, 2, 1, 0, 0,
        );

        {
            macro_rules! do_round {
                (
                    $shuffle_right: expr, $shuffle_left: expr,
                    $shift_right: expr, $shift_left: expr,
                    $src: expr, $dst: expr $(,)?
                ) => {{
                    let mut i = 44;
                    while i != 0 {
                        i -= 1;
                        let old_right = $src[i * 2 + 1];
                        let old_left = $src[i * 2 + 0];
                        let right_mask = _mm512_test_epi16_mask(old_right, old_right);
                        let left_mask = _mm512_test_epi16_mask(old_left, old_left);
                        let mut right = old_right;
                        right =
                            _mm512_max_epu16(right, _mm512_permutexvar_epi16($shift_right, right));
                        right = _mm512_max_epu16(
                            right,
                            _mm512_permutex2var_epi16(old_right, $shuffle_left, old_left),
                        );
                        let mut left = old_left;
                        left = _mm512_max_epu16(
                            left,
                            _mm512_permutex2var_epi16(old_left, $shuffle_right, old_right),
                        );
                        left =
                            _mm512_max_epu16(left, _mm512_permutexvar_epi16($shift_left, old_left));
                        if i != 43 {
                            let i = i + 1;
                            let bottom_r = $src[i * 2 + 1];
                            let bottom_l = $src[i * 2 + 0];
                            // TODO backport the smoothing to GLSL
                            right = _mm512_max_epu16(right, bottom_r);
                            right = _mm512_max_epu16(
                                right,
                                _mm512_permutexvar_epi16($shift_right, bottom_r),
                            );
                            right = _mm512_max_epu16(
                                right,
                                _mm512_permutex2var_epi16(bottom_r, $shuffle_left, bottom_l),
                            );
                            left = _mm512_max_epu16(left, bottom_l);
                            left = _mm512_max_epu16(
                                left,
                                _mm512_permutexvar_epi16($shift_right, bottom_l),
                            );
                            left = _mm512_max_epu16(
                                left,
                                _mm512_permutex2var_epi16(bottom_l, $shuffle_left, bottom_r),
                            );
                        }
                        if i != 0 {
                            let i = i - 1;
                            let top_r = $src[i * 2 + 1];
                            let top_l = $src[i * 2 + 0];
                            right = _mm512_max_epu16(right, top_r);
                            right = _mm512_max_epu16(
                                right,
                                _mm512_permutexvar_epi16($shift_right, top_r),
                            );
                            right = _mm512_max_epu16(
                                right,
                                _mm512_permutex2var_epi16(top_r, $shuffle_left, top_l),
                            );
                            left = _mm512_max_epu16(left, top_l);
                            left = _mm512_max_epu16(
                                left,
                                _mm512_permutexvar_epi16($shift_right, top_l),
                            );
                            left = _mm512_max_epu16(
                                left,
                                _mm512_permutex2var_epi16(top_l, $shuffle_left, top_r),
                            );
                        }
                        $dst[i * 2 + 1] = _mm512_maskz_mov_epi16(right_mask, right);
                        $dst[i * 2 + 0] = _mm512_maskz_mov_epi16(left_mask, left);
                    }
                }};
            }
            for _ in 0..5 {
                do_round!(
                    shuffle_right,
                    shuffle_left,
                    shift_right,
                    shift_left,
                    buf_a,
                    buf_b,
                );
                do_round!(
                    shuffle_right,
                    shuffle_left,
                    shift_right,
                    shift_left,
                    buf_b,
                    buf_a,
                );
            }
            do_round!(
                shuffle_right,
                shuffle_left,
                shift_right,
                shift_left,
                buf_a,
                buf_b,
            );
        }

        let mut indices_ok = [_mm512_setzero_epi32(); 176];
        {
            let mut indexer = _mm512_add_epi16(_mm512_set1_epi16(64 * 44 + 1), indexer);
            let mut i = 44;
            while i != 0 {
                i -= 1;
                indexer = _mm512_sub_epi16(indexer, add_half_row);
                let value = buf_b[i * 2 + 1];
                let mask = _mm512_cmpeq_epi16_mask(value, indexer);
                indices_ok[i * 4 + 3] = _mm512_maskz_mov_epi32(
                    (mask >> 16) as u16,
                    _mm512_cvtepi16_epi32(_mm512_extracti64x4_epi64::<1>(value)),
                );
                indices_ok[i * 4 + 2] = _mm512_maskz_mov_epi32(
                    (mask >> 0) as u16,
                    _mm512_cvtepi16_epi32(_mm512_castsi512_si256(value)),
                );
                indexer = _mm512_sub_epi16(indexer, add_half_row);
                let value = buf_b[i * 2 + 0];
                let mask = _mm512_cmpeq_epi16_mask(value, indexer);
                indices_ok[i * 4 + 1] = _mm512_maskz_mov_epi32(
                    (mask >> 16) as u16,
                    _mm512_cvtepi16_epi32(_mm512_extracti64x4_epi64::<1>(value)),
                );
                indices_ok[i * 4 + 0] = _mm512_maskz_mov_epi32(
                    (mask >> 0) as u16,
                    _mm512_cvtepi16_epi32(_mm512_castsi512_si256(value)),
                );
            }
        }

        {
            let slice = (indices_ok.as_mut_ptr() as *mut u8).wrapping_offset(-4);
            let mut i = 44;
            while i != 0 {
                i -= 1;
                let right = buf_b[i * 2 + 1];
                let left = buf_b[i * 2 + 0];
                let right_active = _mm512_test_epi16_mask(right, right);
                let left_active = _mm512_test_epi16_mask(left, left);
                let mut r_mask = 0u32;
                r_mask |= _mm512_mask_cmpneq_epi16_mask(
                    right_active >> 1,
                    right,
                    _mm512_permutexvar_epi16(shift_right, right),
                );
                r_mask |= _mm512_mask_cmpneq_epi16_mask(
                    left_active >> 31 | right_active << 1,
                    right,
                    _mm512_permutex2var_epi16(right, shuffle_left, left),
                );
                let mut l_mask = 0u32;
                l_mask |= _mm512_mask_cmpneq_epi16_mask(
                    left_active << 1,
                    left,
                    _mm512_permutexvar_epi16(shift_left, left),
                );
                l_mask |= _mm512_mask_cmpneq_epi16_mask(
                    right_active << 31 | left_active >> 1,
                    left,
                    _mm512_permutex2var_epi16(left, shuffle_right, right),
                );
                if i != 43 {
                    let i = i + 1;
                    let bottom_r = buf_b[i * 2 + 1];
                    let bottom_l = buf_b[i * 2 + 0];
                    let bottom_r_active = _mm512_test_epi16_mask(bottom_r, bottom_r);
                    let bottom_l_active = _mm512_test_epi16_mask(bottom_l, bottom_l);
                    // TODO backport the smoothing to GLSL
                    r_mask |= _mm512_mask_cmpneq_epi16_mask(bottom_r_active, right, bottom_r);
                    r_mask |= _mm512_mask_cmpneq_epi16_mask(
                        bottom_r_active >> 1,
                        right,
                        _mm512_permutexvar_epi16(shift_right, bottom_r),
                    );
                    r_mask |= _mm512_mask_cmpneq_epi16_mask(
                        bottom_l_active >> 31 | bottom_r_active << 1,
                        right,
                        _mm512_permutex2var_epi16(bottom_r, shuffle_left, bottom_l),
                    );
                    l_mask |= _mm512_mask_cmpneq_epi16_mask(bottom_l_active, left, bottom_l);
                    l_mask |= _mm512_mask_cmpneq_epi16_mask(
                        bottom_l_active << 1,
                        left,
                        _mm512_permutexvar_epi16(shift_left, bottom_l),
                    );
                    l_mask |= _mm512_mask_cmpneq_epi16_mask(
                        bottom_r_active << 31 | bottom_l_active >> 1,
                        left,
                        _mm512_permutex2var_epi16(bottom_l, shuffle_right, bottom_r),
                    );
                }
                if i != 0 {
                    let i = i - 1;
                    let top_r = buf_b[i * 2 + 1];
                    let top_l = buf_b[i * 2 + 0];
                    let top_r_active = _mm512_test_epi16_mask(top_r, top_r);
                    let top_l_active = _mm512_test_epi16_mask(top_l, top_l);
                    r_mask |= _mm512_mask_cmpneq_epi16_mask(top_r_active, right, top_r);
                    r_mask |= _mm512_mask_cmpneq_epi16_mask(
                        top_r_active >> 1,
                        right,
                        _mm512_permutexvar_epi16(shift_right, top_r),
                    );
                    r_mask |= _mm512_mask_cmpneq_epi16_mask(
                        top_l_active >> 31 | top_r_active << 1,
                        right,
                        _mm512_permutex2var_epi16(top_r, shuffle_left, top_l),
                    );
                    l_mask |= _mm512_mask_cmpneq_epi16_mask(top_l_active, left, top_l);
                    l_mask |= _mm512_mask_cmpneq_epi16_mask(
                        top_l_active << 1,
                        left,
                        _mm512_permutexvar_epi16(shift_left, top_l),
                    );
                    l_mask |= _mm512_mask_cmpneq_epi16_mask(
                        top_r_active << 31 | top_l_active >> 1,
                        left,
                        _mm512_permutex2var_epi16(top_l, shuffle_right, top_r),
                    );
                }
                r_mask &= right_active;
                l_mask &= left_active;
                let right_right = _mm512_cvtepi16_epi32(_mm512_extracti64x4_epi64::<1>(right));
                let right_left = _mm512_cvtepi16_epi32(_mm512_castsi512_si256(right));
                let left_right = _mm512_cvtepi16_epi32(_mm512_extracti64x4_epi64::<1>(left));
                let left_left = _mm512_cvtepi16_epi32(_mm512_castsi512_si256(left));
                _mm512_mask_i32scatter_epi32::<4>(slice, (r_mask >> 16) as u16, right_right, zero);
                _mm512_mask_i32scatter_epi32::<4>(slice, (r_mask >> 0) as u16, right_left, zero);
                _mm512_mask_i32scatter_epi32::<4>(slice, (l_mask >> 16) as u16, left_right, zero);
                _mm512_mask_i32scatter_epi32::<4>(slice, (l_mask >> 0) as u16, left_left, zero);
            }
        }

        let mut indices = _mm512_setzero_epi32();
        {
            let mut i = 176;
            while i != 0 {
                i -= 1;
                let row = _mm512_cvtepi32_epi16(indices_ok[i]);
                indices = _mm512_inserti64x4::<1>(indices, row);
                let mask = _mm512_test_epi16_mask(indices, indices);
                indices = _mm512_maskz_compress_epi16(mask, indices);
            }
        }
        let result = 32 - _mm512_test_epi16_mask(indices, indices).leading_zeros();
        if result > 10 {
            return 0;
        }
        (
            _mm512_cvtepi16_epi32(_mm512_castsi512_si256(indices)),
            result,
        )
    };

    let mut col_y = [_mm512_setzero_epi32(); 64];
    let mut col_w = [_mm512_setzero_epi32(); 64];
    {
        let indices_higher = _mm512_permutexvar_epi32(
            _mm512_castsi256_si512(_mm256_set_epi32(0, 0, 0, 9, 8, 7, 6, 5)),
            indices,
        );
        let indices_lower = indices;
        let indices_selectable = _mm512_set_epi32(0, 0, 0, 0, 10, 5, 9, 4, 8, 3, 7, 2, 6, 1, 0, 0);
        let offset = _mm512_set1_epi32(32);
        let shift_1 = _mm_set_epi64x(0, 1);
        let indices_only = _mm512_set1_epi32(0b11111);
        macro_rules! to_indices {
            (
                $indices_higher: expr, $indices_lower: expr, $indices_selectable: expr,
                $offset: expr, $indices_only: expr, $add_1: expr, $shift_1: expr,
                $row: expr, $addend: expr $(,)?
            ) => {{
                let row = _mm512_cvtepi16_epi32($row);
                let right = _mm512_extracti64x4_epi64::<1>(row);
                let r_h_i = _mm512_inserti64x4::<1>($indices_higher, right);
                let r_high = _mm512_conflict_epi32(r_h_i);
                let r_l_i = _mm512_inserti64x4::<1>($indices_lower, right);
                let r_low = _mm512_conflict_epi32(r_l_i);
                let left = _mm512_castsi512_si256(row);
                let l_h_i = _mm512_inserti64x4::<1>($indices_higher, left);
                let l_high = _mm512_conflict_epi32(l_h_i);
                let l_l_i = _mm512_inserti64x4::<1>($indices_lower, left);
                let l_low = _mm512_conflict_epi32(l_l_i);
                let high = _mm512_inserti64x4::<0>(r_high, _mm512_extracti64x4_epi64::<1>(l_high));
                let high = _mm512_lzcnt_epi32(_mm512_and_epi32($indices_only, high));
                let high = _mm512_sll_epi32(_mm512_sub_epi32($offset, high), $shift_1);
                let high = _mm512_add_epi32(high, $add_1);
                let low = _mm512_inserti64x4::<0>(r_low, _mm512_extracti64x4_epi64::<1>(l_low));
                let low = _mm512_lzcnt_epi32(_mm512_and_epi32($indices_only, low));
                let low = _mm512_sll_epi32(_mm512_sub_epi32($offset, low), $shift_1);
                let gross = _mm512_max_epi32(high, low);
                let gross = _mm512_permutexvar_epi32(gross, $indices_selectable);
                let net = _mm512_add_epi32(gross, $addend);
                net
            }};
        }
        let rri = _mm512_set_epi32(
            0x3F0, 0x3E0, 0x3D0, 0x3C0, 0x3B0, 0x3A0, 0x390, 0x380, 0x370, 0x360, 0x350, 0x340,
            0x330, 0x320, 0x310, 0x300,
        );
        let rli = _mm512_set_epi32(
            0x2F0, 0x2E0, 0x2D0, 0x2C0, 0x2B0, 0x2A0, 0x290, 0x280, 0x270, 0x260, 0x250, 0x240,
            0x230, 0x220, 0x210, 0x200,
        );
        let lri = _mm512_set_epi32(
            0x1F0, 0x1E0, 0x1D0, 0x1C0, 0x1B0, 0x1A0, 0x190, 0x180, 0x170, 0x160, 0x150, 0x140,
            0x130, 0x120, 0x110, 0x100,
        );
        let lli = _mm512_set_epi32(
            0x0F0, 0x0E0, 0x0D0, 0x0C0, 0x0B0, 0x0A0, 0x090, 0x080, 0x070, 0x060, 0x050, 0x040,
            0x030, 0x020, 0x010, 0x000,
        );
        let mut i = 44;
        let mut y = _mm512_set1_epi32(44);
        let debug_col_y = &mut col_y;
        let debug_col_w = &mut col_w;
        let col_y = debug_col_y.as_mut_ptr() as *mut u8;
        let col_w = debug_col_w.as_mut_ptr() as *mut u8;
        while i != 0 {
            i -= 1;
            y = _mm512_sub_epi32(y, add_1);
            let w = inverteds[i];
            let row = buf_b[i * 2 + 1];
            let rrw = _mm512_cvtepi8_epi32(_mm512_extracti32x4_epi32::<3>(w));
            let rry = _mm512_mullo_epi32(rrw, y);
            let rri = to_indices!(
                indices_higher,
                indices_lower,
                indices_selectable,
                offset,
                indices_only,
                add_1,
                shift_1,
                _mm512_extracti64x4_epi64::<1>(row),
                rri,
            );
            _mm512_i32scatter_epi32::<4>(
                col_w,
                rri,
                _mm512_add_epi32(rrw, _mm512_i32gather_epi32::<4>(rri, col_w)),
            );
            _mm512_i32scatter_epi32::<4>(
                col_y,
                rri,
                _mm512_add_epi32(rry, _mm512_i32gather_epi32::<4>(rri, col_y)),
            );
            let rlw = _mm512_cvtepi8_epi32(_mm512_extracti32x4_epi32::<2>(w));
            let rly = _mm512_mullo_epi32(rlw, y);
            let rli = to_indices!(
                indices_higher,
                indices_lower,
                indices_selectable,
                offset,
                indices_only,
                add_1,
                shift_1,
                _mm512_castsi512_si256(row),
                rli,
            );
            _mm512_i32scatter_epi32::<4>(
                col_w,
                rli,
                _mm512_add_epi32(rlw, _mm512_i32gather_epi32::<4>(rli, col_w)),
            );
            _mm512_i32scatter_epi32::<4>(
                col_y,
                rli,
                _mm512_add_epi32(rly, _mm512_i32gather_epi32::<4>(rli, col_y)),
            );
            let row = buf_b[i * 2 + 0];
            let lrw = _mm512_cvtepi8_epi32(_mm512_extracti32x4_epi32::<1>(w));
            let lry = _mm512_mullo_epi32(lrw, y);
            let lri = to_indices!(
                indices_higher,
                indices_lower,
                indices_selectable,
                offset,
                indices_only,
                add_1,
                shift_1,
                _mm512_extracti64x4_epi64::<1>(row),
                lri,
            );
            _mm512_i32scatter_epi32::<4>(
                col_w,
                lri,
                _mm512_add_epi32(lrw, _mm512_i32gather_epi32::<4>(lri, col_w)),
            );
            _mm512_i32scatter_epi32::<4>(
                col_y,
                lri,
                _mm512_add_epi32(lry, _mm512_i32gather_epi32::<4>(lri, col_y)),
            );
            let llw = _mm512_cvtepi8_epi32(_mm512_castsi512_si128(w));
            let lly = _mm512_mullo_epi32(llw, y);
            let lli = to_indices!(
                indices_higher,
                indices_lower,
                indices_selectable,
                offset,
                indices_only,
                add_1,
                shift_1,
                _mm512_castsi512_si256(row),
                lli,
            );
            _mm512_i32scatter_epi32::<4>(
                col_w,
                lli,
                _mm512_add_epi32(llw, _mm512_i32gather_epi32::<4>(lli, col_w)),
            );
            _mm512_i32scatter_epi32::<4>(
                col_y,
                lli,
                _mm512_add_epi32(lly, _mm512_i32gather_epi32::<4>(lli, col_y)),
            );
        }
    }

    let mut x = _mm512_setzero_epi32();
    let mut y = _mm512_setzero_epi32();
    {
        let mut w = _mm512_setzero_epi32();
        let mut i = 64;
        let mut col_i = _mm512_set1_epi32(64);
        while i != 0 {
            i -= 1;
            col_i = _mm512_sub_epi32(col_i, add_1);
            y = _mm512_add_epi32(y, col_y[i]);
            let col = col_w[i];
            w = _mm512_add_epi32(w, col);
            x = _mm512_add_epi32(x, _mm512_mullo_epi32(col, col_i));
        }
        let w = _mm512_cvtepu32_ps(w);
        y = _mm512_sub_epi32(
            _mm512_set1_epi32(1823),
            _mm512_cvtps_epu32(_mm512_div_ps(
                _mm512_cvtepu32_ps(y),
                _mm512_mul_ps(w, _mm512_set1_ps(43f32 / 1823f32)),
            )),
        );
        x = _mm512_sub_epi32(
            _mm512_set1_epi32(2735),
            _mm512_cvtps_epu32(_mm512_div_ps(
                _mm512_cvtepu32_ps(x),
                _mm512_mul_ps(w, _mm512_set1_ps(63f32 / 2735f32)),
            )),
        );
    }

    let x = std::slice::from_raw_parts(&x as *const __m512i as *const u32, 16);
    let y = std::slice::from_raw_parts(&y as *const __m512i as *const u32, 16);
    {
        let mut i = 10;
        while i != 0 {
            i -= 1;
            results[i] = (x[i + 1], y[i + 1]);
        }
    }
    result
}

pub fn process_heatmap(data: &[u8; 2816], results: &mut [(u32, u32); 10]) -> usize {
    unsafe { process_heatmap_internal(data, results) as usize }
}
