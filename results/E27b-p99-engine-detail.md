# E27b — every cell

*Generated with `results/E27b-comparator.md`, from the same point files. One row per (pair, metric) per point: medians and MADs over the ten measured runs, pooled MAD, floor (3 × pooled MAD of the warm-up runs), relative change (B − A)/A, MADs apart, verdict and, when refused, why.*

## `multi`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 293.7 | 7.34 | 685.5 | 17.8 | 13.6 | 100.5 | +133.4% | 28.8 | **N better** |
| N vs P+ | q1_p99_us | 680.8 | 54.6 | 1491 | 120.7 | 93.7 | 32.0 | +119.1% | 8.65 | **N better** |
| N vs P+ | q2_p50_us | 8596 | 276.9 | 530.4 | 18.9 | 196.2 | 494.7 | -93.8% | 41.1 | **P+ better** |
| N vs P+ | q2_p99_us | 21405 | 920.3 | 1225 | 149.8 | 659.3 | 3954 | -94.3% | 30.6 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 245.5 | 11.0 | 2119 | 98.1 | 69.8 | 102.6 | +763.2% | 26.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 6074 | 166.0 | 7683 | 625.3 | 457.5 | 2869 | +26.5% | 3.52 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 17454 | 183.0 | 1804 | 100.9 | 147.8 | 903.5 | -89.7% | 105.9 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 46422 | 3926 | 6406 | 766.9 | 2828 | 4718 | -86.2% | 14.1 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 268.6 | 0.52 | 857.7 | 12.4 | 8.80 | 20.5 | +219.3% | 67.0 | **N better** |
| N vs P+ | pss_mib | 38.5 | 0.06 | 45.6 | 0.03 | 0.05 | 0.16 | +18.4% | 149.9 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 26040 | 46.8 | -4748 | 32.0 | 40.1 | 1777 | -118.2% | 768.0 | **P+ better** |
| N vs H3 | q1_p50_us | 293.7 | 7.34 | 296.9 | 20.4 | 15.3 | 83.6 | +1.1% | 0.21 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 680.8 | 54.6 | 762.0 | 38.4 | 47.2 | 654.9 | +11.9% | 1.72 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 8596 | 276.9 | 287.7 | 16.4 | 196.1 | 495.9 | -96.7% | 42.4 | **H3 better** |
| N vs H3 | q2_p99_us | 21405 | 920.3 | 680.7 | 42.5 | 651.4 | 3954 | -96.8% | 31.8 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 245.5 | 11.0 | 556.4 | 21.3 | 16.9 | 75.3 | +126.7% | 18.4 | **N better** |
| N vs H3 | c4_q1_p99_us | 6074 | 166.0 | 1722 | 80.6 | 130.5 | 1809 | -71.7% | 33.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 17454 | 183.0 | 539.2 | 14.6 | 129.8 | 903.4 | -96.9% | 130.3 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 46422 | 3926 | 1773 | 165.5 | 2778 | 4774 | -96.2% | 16.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | view_state_bytes_per_key_read | 268.6 | 0.52 | 110.5 | 0.14 | 0.38 | 3.82 | -58.9% | 416.6 | **H3 better** |
| N vs H3 | pss_mib | 38.5 | 0.06 | 72.6 | 0.16 | 0.12 | 0.12 | +88.4% | 286.6 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 26040 | 46.8 | 11851 | 70.6 | 59.9 | 1757 | -54.5% | 236.8 | **H3 better** |
| P+ vs H3 | q1_p50_us | 685.5 | 17.8 | 296.9 | 20.4 | 19.1 | 113.2 | -56.7% | 20.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1491 | 120.7 | 762.0 | 38.4 | 89.6 | 655.7 | -48.9% | 8.14 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 530.4 | 18.9 | 287.7 | 16.4 | 17.7 | 67.2 | -45.8% | 13.7 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1225 | 149.8 | 680.7 | 42.5 | 110.1 | 134.7 | -44.4% | 4.95 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2119 | 98.1 | 556.4 | 21.3 | 71.0 | 113.7 | -73.7% | 22.0 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 7683 | 625.3 | 1722 | 80.6 | 445.8 | 3310 | -77.6% | 13.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1804 | 100.9 | 539.2 | 14.6 | 72.1 | 22.8 | -70.1% | 17.5 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6406 | 766.9 | 1773 | 165.5 | 554.8 | 799.5 | -72.3% | 8.35 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | view_state_bytes_per_key_read | 857.7 | 12.4 | 110.5 | 0.14 | 8.79 | 20.1 | -87.1% | 85.0 | **H3 better** |
| P+ vs H3 | pss_mib | 45.6 | 0.03 | 72.6 | 0.16 | 0.11 | 0.14 | +59.1% | 239.1 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -4748 | 32.0 | 11851 | 70.6 | 54.8 | 412.4 | -349.6% | 302.7 | **P+ better** |

## `multi`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 295.1 | 5.85 | 661.1 | 29.0 | 20.9 | 62.5 | +124.0% | 17.5 | **N better** |
| N vs P+ | q1_p99_us | 700.9 | 45.6 | 1350 | 145.1 | 107.5 | 1134 | +92.6% | 6.03 | BELOW FLOOR |
| N vs P+ | q2_p50_us | 8568 | 451.2 | 506.2 | 22.3 | 319.4 | 105.6 | -94.1% | 25.2 | **P+ better** |
| N vs P+ | q2_p99_us | 21647 | 2027 | 1024 | 87.2 | 1435 | 2992 | -95.3% | 14.4 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 241.1 | 10.1 | 2117 | 88.2 | 62.8 | 234.1 | +777.9% | 29.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 6544 | 447.1 | 6907 | 860.2 | 685.5 | 920.2 | +5.5% | 0.53 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 16743 | 539.7 | 1799 | 100.8 | 388.2 | 1906 | -89.3% | 38.5 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 43879 | 1554 | 5952 | 490.4 | 1152 | 8958 | -86.4% | 32.9 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 266.7 | 0.71 | 790.0 | 33.3 | 23.6 | 30.4 | +196.2% | 22.2 | **N better** |
| N vs P+ | pss_mib | 38.5 | 0.11 | 45.7 | 0.02 | 0.08 | 0.56 | +18.5% | 93.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 26165 | 92.1 | -2019 | 12.0 | 65.7 | 2224 | -107.7% | 429.1 | **P+ better** |
| N vs H3 | q1_p50_us | 295.1 | 5.85 | 302.9 | 20.5 | 15.0 | 24.2 | +2.7% | 0.52 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 700.9 | 45.6 | 737.3 | 70.6 | 59.5 | 155.3 | +5.2% | 0.61 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 8568 | 451.2 | 288.4 | 20.0 | 319.4 | 77.5 | -96.6% | 25.9 | **H3 better** |
| N vs H3 | q2_p99_us | 21647 | 2027 | 664.5 | 53.4 | 1434 | 2916 | -96.9% | 14.6 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 241.1 | 10.1 | 576.2 | 38.1 | 27.9 | 119.6 | +139.0% | 12.0 | **N better** |
| N vs H3 | c4_q1_p99_us | 6544 | 447.1 | 1715 | 153.5 | 334.2 | 1339 | -73.8% | 14.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 16743 | 539.7 | 556.7 | 29.2 | 382.2 | 1908 | -96.7% | 42.4 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 43879 | 1554 | 1671 | 221.2 | 1110 | 8964 | -96.2% | 38.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | view_state_bytes_per_key_read | 266.7 | 0.71 | 108.0 | 0.08 | 0.51 | 1.51 | -59.5% | 312.5 | **H3 better** |
| N vs H3 | pss_mib | 38.5 | 0.11 | 72.5 | 0.22 | 0.17 | 0.56 | +88.1% | 198.8 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 26165 | 92.1 | 11718 | 167.3 | 135.1 | 2262 | -55.2% | 107.0 | **H3 better** |
| P+ vs H3 | q1_p50_us | 661.1 | 29.0 | 302.9 | 20.5 | 25.1 | 66.7 | -54.2% | 14.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1350 | 145.1 | 737.3 | 70.6 | 114.1 | 1137 | -45.4% | 5.37 | BELOW FLOOR |
| P+ vs H3 | q2_p50_us | 506.2 | 22.3 | 288.4 | 20.0 | 21.2 | 87.1 | -43.0% | 10.3 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1024 | 87.2 | 664.5 | 53.4 | 72.3 | 697.0 | -35.1% | 4.98 | BELOW FLOOR |
| P+ vs H3 | c4_q1_p50_us | 2117 | 88.2 | 576.2 | 38.1 | 67.9 | 262.8 | -72.8% | 22.7 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6907 | 860.2 | 1715 | 153.5 | 617.8 | 1530 | -75.2% | 8.40 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1799 | 100.8 | 556.7 | 29.2 | 74.2 | 105.6 | -69.1% | 16.7 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5952 | 490.4 | 1671 | 221.2 | 380.4 | 1772 | -71.9% | 11.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | view_state_bytes_per_key_read | 790.0 | 33.3 | 108.0 | 0.08 | 23.6 | 30.4 | -86.3% | 28.9 | **H3 better** |
| P+ vs H3 | pss_mib | 45.7 | 0.02 | 72.5 | 0.22 | 0.15 | 0.02 | +58.7% | 174.8 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2019 | 12.0 | 11718 | 167.3 | 118.6 | 412.6 | -680.4% | 115.8 | **P+ better** |

## `multi`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 277.9 | 12.6 | 708.2 | 22.8 | 18.5 | 122.6 | +154.9% | 23.3 | **N better** |
| N vs P+ | q1_p99_us | 677.2 | 49.8 | 1334 | 68.8 | 60.0 | 702.2 | +96.9% | 10.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 8384 | 287.1 | 541.4 | 26.2 | 203.8 | 129.7 | -93.5% | 38.5 | **P+ better** |
| N vs P+ | q2_p99_us | 21737 | 1098 | 1170 | 178.1 | 786.4 | 2371 | -94.6% | 26.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 234.5 | 7.25 | 1927 | 30.7 | 22.3 | 23.8 | +721.5% | 76.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 6326 | 545.9 | 6255 | 1225 | 948.5 | 1732 | -1.1% | 0.08 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 16555 | 466.1 | 1639 | 41.1 | 330.9 | 244.1 | -90.1% | 45.1 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 39003 | 2147 | 5366 | 352.9 | 1539 | 2674 | -86.2% | 21.9 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 265.7 | 0.20 | 825.4 | 18.9 | 13.4 | 22.5 | +210.7% | 41.8 | **N better** |
| N vs P+ | pss_mib | 39.0 | 0.00 | 45.8 | 0.01 | 0.01 | 0.76 | +17.3% | 815.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 26427 | 10.0 | -1867 | 10.0 | 10.0 | 1873 | -107.1% | 2825 | **P+ better** |
| N vs H3 | q1_p50_us | 277.9 | 12.6 | 281.7 | 7.66 | 10.4 | 30.9 | +1.4% | 0.37 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 677.2 | 49.8 | 709.8 | 76.3 | 64.4 | 164.9 | +4.8% | 0.51 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 8384 | 287.1 | 272.5 | 5.37 | 203.0 | 33.5 | -96.7% | 40.0 | **H3 better** |
| N vs H3 | q2_p99_us | 21737 | 1098 | 627.7 | 62.6 | 777.5 | 2303 | -97.1% | 27.2 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 234.5 | 7.25 | 523.5 | 19.5 | 14.7 | 43.9 | +123.2% | 19.6 | **N better** |
| N vs H3 | c4_q1_p99_us | 6326 | 545.9 | 1856 | 142.7 | 399.0 | 458.8 | -70.7% | 11.2 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 16555 | 466.1 | 521.1 | 17.7 | 329.8 | 161.5 | -96.9% | 48.6 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 39003 | 2147 | 1756 | 150.3 | 1522 | 2674 | -95.5% | 24.5 | **H3 better** |
| N vs H3 | view_state_bytes_per_key_read | 265.7 | 0.20 | 107.1 | 0.26 | 0.23 | 7.71 | -59.7% | 689.8 | **H3 better** |
| N vs H3 | pss_mib | 39.0 | 0.00 | 72.3 | 0.04 | 0.03 | 0.52 | +85.5% | 1271 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 26427 | 10.0 | 11674 | 31.3 | 23.2 | 1815 | -55.8% | 635.3 | **H3 better** |
| P+ vs H3 | q1_p50_us | 708.2 | 22.8 | 281.7 | 7.66 | 17.0 | 121.2 | -60.2% | 25.0 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1334 | 68.8 | 709.8 | 76.3 | 72.6 | 703.3 | -46.8% | 8.59 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 541.4 | 26.2 | 272.5 | 5.37 | 18.9 | 129.4 | -49.7% | 14.2 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1170 | 178.1 | 627.7 | 62.6 | 133.5 | 571.4 | -46.4% | 4.06 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 1927 | 30.7 | 523.5 | 19.5 | 25.7 | 43.5 | -72.8% | 54.6 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6255 | 1225 | 1856 | 142.7 | 872.3 | 1786 | -70.3% | 5.04 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1639 | 41.1 | 521.1 | 17.7 | 31.7 | 206.4 | -68.2% | 35.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5366 | 352.9 | 1756 | 150.3 | 271.2 | 57.9 | -67.3% | 13.3 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 825.4 | 18.9 | 107.1 | 0.26 | 13.4 | 21.2 | -87.0% | 53.6 | **H3 better** |
| P+ vs H3 | pss_mib | 45.8 | 0.01 | 72.3 | 0.04 | 0.03 | 0.55 | +58.1% | 966.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -1867 | 10.0 | 11674 | 31.3 | 23.2 | 460.9 | -725.3% | 583.1 | **P+ better** |

## `multi`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 290.2 | 6.14 | 799.4 | 40.5 | 29.0 | 48.6 | +175.4% | 17.6 | **N better** |
| N vs P+ | q1_p99_us | 693.4 | 17.9 | 1805 | 132.6 | 94.6 | 210.9 | +160.3% | 11.7 | **N better** |
| N vs P+ | q2_p50_us | 8645 | 117.9 | 622.6 | 44.3 | 89.1 | 9.82 | -92.8% | 90.1 | **P+ better** |
| N vs P+ | q2_p99_us | 22040 | 1916 | 1332 | 192.7 | 1362 | 1159 | -94.0% | 15.2 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 252.9 | 18.5 | 2382 | 69.8 | 51.1 | 109.8 | +842.0% | 41.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 6369 | 382.0 | 8689 | 1190 | 883.8 | 914.2 | +36.4% | 2.62 | no difference |
| N vs P+ | c4_q2_p50_us | 17098 | 385.5 | 2023 | 20.2 | 272.9 | 832.0 | -88.2% | 55.2 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 42922 | 2068 | 7423 | 673.8 | 1538 | 2793 | -82.7% | 23.1 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 269.3 | 0.50 | 811.8 | 46.5 | 32.9 | 24.4 | +201.4% | 16.5 | **N better** |
| N vs P+ | pss_mib | 38.8 | 0.13 | 45.8 | 0.00 | 0.09 | 0.29 | +18.1% | 76.9 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 26205 | 136.3 | -1989 | 1.55 | 96.4 | 1423 | -107.6% | 292.5 | **P+ better** |
| N vs H3 | q1_p50_us | 290.2 | 6.14 | 291.8 | 9.27 | 7.86 | 39.0 | +0.5% | 0.20 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 693.4 | 17.9 | 634.3 | 63.2 | 46.4 | 164.2 | -8.5% | 1.27 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 8645 | 117.9 | 281.0 | 6.84 | 83.5 | 27.7 | -96.7% | 100.1 | **H3 better** |
| N vs H3 | q2_p99_us | 22040 | 1916 | 643.2 | 58.4 | 1356 | 1134 | -97.1% | 15.8 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 252.9 | 18.5 | 546.5 | 29.1 | 24.4 | 20.7 | +116.1% | 12.0 | **N better** |
| N vs H3 | c4_q1_p99_us | 6369 | 382.0 | 1618 | 119.0 | 282.9 | 341.8 | -74.6% | 16.8 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 17098 | 385.5 | 543.2 | 27.1 | 273.2 | 829.4 | -96.8% | 60.6 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 42922 | 2068 | 1641 | 119.9 | 1465 | 1834 | -96.2% | 28.2 | **H3 better** |
| N vs H3 | view_state_bytes_per_key_read | 269.3 | 0.50 | 107.4 | 0.14 | 0.37 | 1.36 | -60.1% | 438.3 | **H3 better** |
| N vs H3 | pss_mib | 38.8 | 0.13 | 72.4 | 0.18 | 0.16 | 0.24 | +86.9% | 212.7 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 26205 | 136.3 | 11557 | 134.4 | 135.4 | 1630 | -55.9% | 108.2 | **H3 better** |
| P+ vs H3 | q1_p50_us | 799.4 | 40.5 | 291.8 | 9.27 | 29.4 | 58.4 | -63.5% | 17.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1805 | 132.6 | 634.3 | 63.2 | 103.9 | 250.6 | -64.9% | 11.3 | **H3 better** |
| P+ vs H3 | q2_p50_us | 622.6 | 44.3 | 281.0 | 6.84 | 31.7 | 26.6 | -54.9% | 10.8 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1332 | 192.7 | 643.2 | 58.4 | 142.4 | 356.5 | -51.7% | 4.84 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2382 | 69.8 | 546.5 | 29.1 | 53.5 | 108.2 | -77.1% | 34.3 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 8689 | 1190 | 1618 | 119.0 | 845.7 | 912.1 | -81.4% | 8.36 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 2023 | 20.2 | 543.2 | 27.1 | 23.9 | 80.4 | -73.1% | 61.9 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7423 | 673.8 | 1641 | 119.9 | 483.9 | 2114 | -77.9% | 11.9 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 811.8 | 46.5 | 107.4 | 0.14 | 32.9 | 24.3 | -86.8% | 21.4 | **H3 better** |
| P+ vs H3 | pss_mib | 45.8 | 0.00 | 72.4 | 0.18 | 0.13 | 0.26 | +58.3% | 205.9 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -1989 | 1.55 | 11557 | 134.4 | 95.0 | 794.9 | -681.0% | 142.5 | **P+ better** |

## `multi`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 289.5 | 5.50 | 772.9 | 30.1 | 21.6 | 78.9 | +166.9% | 22.3 | **N better** |
| N vs P+ | q1_p99_us | 700.9 | 38.2 | 1844 | 192.8 | 139.0 | 134.1 | +163.1% | 8.23 | **N better** |
| N vs P+ | q2_p50_us | 8800 | 192.2 | 607.6 | 29.0 | 137.4 | 61.3 | -93.1% | 59.6 | **P+ better** |
| N vs P+ | q2_p99_us | 21437 | 1709 | 1628 | 236.1 | 1220 | 832.9 | -92.4% | 16.2 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 249.6 | 15.2 | 2170 | 81.5 | 58.6 | 161.0 | +769.2% | 32.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 6436 | 345.5 | 7107 | 1282 | 939.1 | 676.5 | +10.4% | 0.71 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 16915 | 613.7 | 1897 | 70.8 | 436.8 | 657.4 | -88.8% | 34.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 43755 | 3414 | 6910 | 1523 | 2643 | 3715 | -84.2% | 13.9 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 270.7 | 0.37 | 830.4 | 6.24 | 4.42 | 46.9 | +206.8% | 126.7 | **N better** |
| N vs P+ | pss_mib | 39.4 | 0.00 | 45.8 | 0.02 | 0.02 | 0.14 | +16.3% | 364.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 26851 | 0.00 | -2000 | 26.1 | 18.4 | 1675 | -107.4% | 1565 | **P+ better** |
| N vs H3 | q1_p50_us | 289.5 | 5.50 | 288.1 | 7.17 | 6.39 | 13.0 | -0.5% | 0.22 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 700.9 | 38.2 | 793.3 | 49.7 | 44.3 | 105.9 | +13.2% | 2.08 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 8800 | 192.2 | 275.9 | 5.94 | 135.9 | 17.1 | -96.9% | 62.7 | **H3 better** |
| N vs H3 | q2_p99_us | 21437 | 1709 | 667.3 | 82.4 | 1210 | 828.2 | -96.9% | 17.2 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 249.6 | 15.2 | 523.9 | 36.2 | 27.8 | 92.2 | +109.9% | 9.87 | **N better** |
| N vs H3 | c4_q1_p99_us | 6436 | 345.5 | 1862 | 317.0 | 331.6 | 1441 | -71.1% | 13.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 16915 | 613.7 | 535.4 | 36.0 | 434.7 | 660.0 | -96.8% | 37.7 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 43755 | 3414 | 1711 | 340.3 | 2426 | 3609 | -96.1% | 17.3 | **H3 better** |
| N vs H3 | view_state_bytes_per_key_read | 270.7 | 0.37 | 109.5 | 0.25 | 0.31 | 4.71 | -59.5% | 514.6 | **H3 better** |
| N vs H3 | pss_mib | 39.4 | 0.00 | 72.4 | 0.12 | 0.09 | 0.48 | +83.9% | 382.5 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 26851 | 0.00 | 11617 | 95.5 | 67.5 | 1704 | -56.7% | 225.7 | **H3 better** |
| P+ vs H3 | q1_p50_us | 772.9 | 30.1 | 288.1 | 7.17 | 21.9 | 78.2 | -62.7% | 22.2 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1844 | 192.8 | 793.3 | 49.7 | 140.8 | 140.4 | -57.0% | 7.46 | **H3 better** |
| P+ vs H3 | q2_p50_us | 607.6 | 29.0 | 275.9 | 5.94 | 20.9 | 60.2 | -54.6% | 15.8 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1628 | 236.1 | 667.3 | 82.4 | 176.8 | 93.3 | -59.0% | 5.44 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2170 | 81.5 | 523.9 | 36.2 | 63.1 | 181.0 | -75.9% | 26.1 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 7107 | 1282 | 1862 | 317.0 | 934.0 | 1477 | -73.8% | 5.62 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1897 | 70.8 | 535.4 | 36.0 | 56.2 | 92.7 | -71.8% | 24.2 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6910 | 1523 | 1711 | 340.3 | 1103 | 930.0 | -75.2% | 4.71 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 830.4 | 6.24 | 109.5 | 0.25 | 4.42 | 46.6 | -86.8% | 163.3 | **H3 better** |
| P+ vs H3 | pss_mib | 45.8 | 0.02 | 72.4 | 0.12 | 0.09 | 0.49 | +58.1% | 302.0 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2000 | 26.1 | 11617 | 95.5 | 70.0 | 473.1 | -680.8% | 194.6 | **P+ better** |

## `multi`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 776.8 | 92.4 | 775.3 | 31.7 | 69.1 | 141.2 | -0.2% | 0.02 | BELOW FLOOR |
| N vs P+ | q1_p99_us | 1653 | 150.5 | 1705 | 135.5 | 143.2 | 632.1 | +3.2% | 0.37 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 102688 | 1816 | 611.7 | 31.3 | 1284 | 3173 | -99.4% | 79.5 | **P+ better** |
| N vs P+ | q2_p99_us | 215005 | 6529 | 1168 | 127.8 | 4618 | 8117 | -99.5% | 46.3 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 911.4 | 69.3 | 2175 | 121.7 | 99.0 | 191.5 | +138.6% | 12.8 | **N better** |
| N vs P+ | c4_q1_p99_us | 6637 | 130.3 | 6565 | 711.2 | 511.3 | 1093 | -1.1% | 0.14 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 198777 | 3953 | 1889 | 109.5 | 2796 | 1162 | -99.0% | 70.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 419153 | 5598 | 6566 | 834.6 | 4002 | 41781 | -98.4% | 103.1 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 248.5 | 10.2 | 324.4 | 15.8 | 13.3 | 291.4 | +30.6% | 5.70 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 311.5 | 0.19 | 92.2 | 0.18 | 0.19 | 1.17 | -70.4% | 1178 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 28481 | 2369 | 4426 | 350.3 | 1693 | 29185 | -84.5% | 14.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 776.8 | 92.4 | 509.0 | 41.8 | 71.7 | 168.3 | -34.5% | 3.74 | **H3 better** |
| N vs H3 | q1_p99_us | 1653 | 150.5 | 2212 | 325.0 | 253.3 | 475.1 | +33.8% | 2.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 102688 | 1816 | 473.3 | 29.2 | 1284 | 3174 | -99.5% | 79.6 | **H3 better** |
| N vs H3 | q2_p99_us | 215005 | 6529 | 1665 | 129.8 | 4618 | 8117 | -99.2% | 46.2 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 911.4 | 69.3 | 1115 | 69.7 | 69.5 | 243.3 | +22.3% | 2.93 | BELOW FLOOR |
| N vs H3 | c4_q1_p99_us | 6637 | 130.3 | 3841 | 510.9 | 372.9 | 107.8 | -42.1% | 7.50 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 198777 | 3953 | 1103 | 51.5 | 2795 | 1166 | -99.4% | 70.7 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 419153 | 5598 | 3613 | 235.1 | 3962 | 41762 | -99.1% | 104.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | view_state_bytes_per_key_read | 248.5 | 10.2 | 110.3 | 0.68 | 7.25 | 169.7 | -55.6% | 19.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 311.5 | 0.19 | 110.4 | 0.27 | 0.23 | 1.27 | -64.6% | 865.9 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 28481 | 2369 | 5616 | 389.5 | 1698 | 29534 | -80.3% | 13.5 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 775.3 | 31.7 | 509.0 | 41.8 | 37.1 | 92.3 | -34.3% | 7.18 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1705 | 135.5 | 2212 | 325.0 | 249.0 | 432.9 | +29.7% | 2.03 | no difference |
| P+ vs H3 | q2_p50_us | 611.7 | 31.3 | 473.3 | 29.2 | 30.3 | 130.0 | -22.6% | 4.57 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1168 | 127.8 | 1665 | 129.8 | 128.8 | 128.9 | +42.6% | 3.86 | **P+ better** |
| P+ vs H3 | c4_q1_p50_us | 2175 | 121.7 | 1115 | 69.7 | 99.2 | 248.4 | -48.7% | 10.7 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6565 | 711.2 | 3841 | 510.9 | 619.2 | 1090 | -41.5% | 4.40 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1889 | 109.5 | 1103 | 51.5 | 85.6 | 191.0 | -41.6% | 9.19 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6566 | 834.6 | 3613 | 235.1 | 613.1 | 2481 | -45.0% | 4.82 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | view_state_bytes_per_key_read | 324.4 | 15.8 | 110.3 | 0.68 | 11.2 | 237.1 | -66.0% | 19.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 92.2 | 0.18 | 110.4 | 0.27 | 0.23 | 1.59 | +19.6% | 80.0 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 4426 | 350.3 | 5616 | 389.5 | 370.4 | 4567 | +26.9% | 3.21 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 763.6 | 53.2 | 718.2 | 36.1 | 45.5 | 301.4 | -6.0% | 1.00 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 1632 | 81.4 | 1731 | 156.5 | 124.8 | 372.3 | +6.0% | 0.79 | BELOW FLOOR |
| N vs P+ | q2_p50_us | 102109 | 3003 | 552.7 | 26.5 | 2123 | 4780 | -99.5% | 47.8 | **P+ better** |
| N vs P+ | q2_p99_us | 213094 | 4283 | 1147 | 145.1 | 3030 | 993.6 | -99.5% | 69.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 908.3 | 54.2 | 2040 | 101.0 | 81.1 | 235.3 | +124.6% | 14.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 6706 | 146.0 | 7122 | 557.2 | 407.3 | 149.8 | +6.2% | 1.02 | no difference |
| N vs P+ | c4_q2_p50_us | 202045 | 2764 | 1738 | 76.4 | 1955 | 15061 | -99.1% | 102.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 420214 | 5258 | 5991 | 1017 | 3787 | 3267 | -98.6% | 109.4 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 247.9 | 9.62 | 320.6 | 10.3 | 9.97 | 313.7 | +29.3% | 7.28 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 311.0 | 0.34 | 92.4 | 0.17 | 0.27 | 4.34 | -70.3% | 814.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 28276 | 2276 | 4432 | 337.0 | 1627 | 29915 | -84.3% | 14.7 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 763.6 | 53.2 | 490.4 | 17.7 | 39.6 | 299.8 | -35.8% | 6.90 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 1632 | 81.4 | 1906 | 163.1 | 128.9 | 342.2 | +16.8% | 2.13 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 102109 | 3003 | 463.0 | 10.4 | 2123 | 4780 | -99.5% | 47.9 | **H3 better** |
| N vs H3 | q2_p99_us | 213094 | 4283 | 1739 | 201.9 | 3032 | 757.5 | -99.2% | 69.7 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 908.3 | 54.2 | 1122 | 18.4 | 40.5 | 236.7 | +23.6% | 5.29 | BELOW FLOOR |
| N vs H3 | c4_q1_p99_us | 6706 | 146.0 | 3789 | 341.0 | 262.3 | 951.8 | -43.5% | 11.1 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 202045 | 2764 | 1126 | 35.5 | 1955 | 15063 | -99.4% | 102.8 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 420214 | 5258 | 3833 | 548.5 | 3738 | 2990 | -99.1% | 111.4 | **H3 better** |
| N vs H3 | view_state_bytes_per_key_read | 247.9 | 9.62 | 110.7 | 0.41 | 6.81 | 181.9 | -55.4% | 20.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 311.0 | 0.34 | 111.3 | 0.18 | 0.27 | 4.54 | -64.2% | 731.3 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 28276 | 2276 | 5675 | 446.5 | 1640 | 30331 | -79.9% | 13.8 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 718.2 | 36.1 | 490.4 | 17.7 | 28.5 | 57.6 | -31.7% | 8.01 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1731 | 156.5 | 1906 | 163.1 | 159.8 | 147.2 | +10.1% | 1.10 | no difference |
| P+ vs H3 | q2_p50_us | 552.7 | 26.5 | 463.0 | 10.4 | 20.1 | 86.9 | -16.2% | 4.46 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1147 | 145.1 | 1739 | 201.9 | 175.8 | 647.0 | +51.7% | 3.37 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 2040 | 101.0 | 1122 | 18.4 | 72.6 | 282.6 | -45.0% | 12.6 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 7122 | 557.2 | 3789 | 341.0 | 461.9 | 942.0 | -46.8% | 7.21 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1738 | 76.4 | 1126 | 35.5 | 59.6 | 267.1 | -35.2% | 10.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5991 | 1017 | 3833 | 548.5 | 817.2 | 1558 | -36.0% | 2.64 | no difference |
| P+ vs H3 | view_state_bytes_per_key_read | 320.6 | 10.3 | 110.7 | 0.41 | 7.30 | 255.8 | -65.5% | 28.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 92.4 | 0.17 | 111.3 | 0.18 | 0.18 | 1.91 | +20.4% | 106.8 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 4432 | 337.0 | 5675 | 446.5 | 395.6 | 5037 | +28.0% | 3.14 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 806.6 | 65.0 | 748.7 | 59.1 | 62.1 | 74.4 | -7.2% | 0.93 | BELOW FLOOR |
| N vs P+ | q1_p99_us | 1761 | 143.9 | 1796 | 316.0 | 245.5 | 918.4 | +2.0% | 0.14 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 103762 | 1554 | 571.5 | 44.1 | 1099 | 2168 | -99.4% | 93.9 | **P+ better** |
| N vs P+ | q2_p99_us | 211827 | 4232 | 1364 | 279.7 | 2999 | 21059 | -99.4% | 70.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 928.6 | 21.1 | 2095 | 175.3 | 124.8 | 320.9 | +125.6% | 9.35 | **N better** |
| N vs P+ | c4_q1_p99_us | 6753 | 237.8 | 6382 | 670.4 | 503.0 | 555.4 | -5.5% | 0.74 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 200575 | 2010 | 1830 | 173.9 | 1427 | 491.0 | -99.1% | 139.3 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 418974 | 7692 | 6375 | 724.7 | 5463 | 20557 | -98.5% | 75.5 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 249.9 | 10.4 | 328.9 | 18.0 | 14.7 | 290.9 | +31.6% | 5.38 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 311.2 | 0.15 | 92.2 | 0.18 | 0.17 | 6.35 | -70.4% | 1316 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 28537 | 2363 | 4449 | 344.4 | 1689 | 28923 | -84.4% | 14.3 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 806.6 | 65.0 | 489.4 | 21.8 | 48.5 | 66.0 | -39.3% | 6.54 | **H3 better** |
| N vs H3 | q1_p99_us | 1761 | 143.9 | 2183 | 335.5 | 258.1 | 511.7 | +24.0% | 1.64 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 103762 | 1554 | 472.0 | 29.0 | 1099 | 2163 | -99.5% | 94.0 | **H3 better** |
| N vs H3 | q2_p99_us | 211827 | 4232 | 1777 | 162.7 | 2994 | 21050 | -99.2% | 70.1 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 928.6 | 21.1 | 1099 | 39.8 | 31.9 | 290.5 | +18.3% | 5.33 | BELOW FLOOR |
| N vs H3 | c4_q1_p99_us | 6753 | 237.8 | 3678 | 323.7 | 284.1 | 1491 | -45.5% | 10.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 200575 | 2010 | 1106 | 46.4 | 1422 | 428.3 | -99.4% | 140.3 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 418974 | 7692 | 3639 | 177.1 | 5440 | 20465 | -99.1% | 76.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | view_state_bytes_per_key_read | 249.9 | 10.4 | 110.1 | 0.34 | 7.34 | 179.0 | -55.9% | 19.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 311.2 | 0.15 | 111.0 | 0.28 | 0.22 | 2.08 | -64.3% | 892.0 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 28537 | 2363 | 5703 | 449.5 | 1701 | 29273 | -80.0% | 13.4 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 748.7 | 59.1 | 489.4 | 21.8 | 44.5 | 66.6 | -34.6% | 5.82 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1796 | 316.0 | 2183 | 335.5 | 325.9 | 1019 | +21.6% | 1.19 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 571.5 | 44.1 | 472.0 | 29.0 | 37.3 | 149.4 | -17.4% | 2.66 | BELOW FLOOR |
| P+ vs H3 | q2_p99_us | 1364 | 279.7 | 1777 | 162.7 | 228.8 | 634.3 | +30.2% | 1.80 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 2095 | 175.3 | 1099 | 39.8 | 127.1 | 401.1 | -47.6% | 7.84 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6382 | 670.4 | 3678 | 323.7 | 526.4 | 1590 | -42.4% | 5.14 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1830 | 173.9 | 1106 | 46.4 | 127.3 | 287.5 | -39.6% | 5.69 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6375 | 724.7 | 3639 | 177.1 | 527.5 | 3023 | -42.9% | 5.19 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | view_state_bytes_per_key_read | 328.9 | 18.0 | 110.1 | 0.34 | 12.7 | 229.4 | -66.5% | 17.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 92.2 | 0.18 | 111.0 | 0.28 | 0.23 | 6.66 | +20.4% | 80.7 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 4449 | 344.4 | 5703 | 449.5 | 400.4 | 4990 | +28.2% | 3.13 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 775.8 | 79.9 | 836.1 | 105.6 | 93.7 | 155.5 | +7.8% | 0.64 | BELOW FLOOR |
| N vs P+ | q1_p99_us | 1835 | 149.8 | 1910 | 381.8 | 290.0 | 302.3 | +4.1% | 0.26 | BELOW FLOOR |
| N vs P+ | q2_p50_us | 103967 | 1051 | 648.5 | 100.2 | 746.3 | 3206 | -99.4% | 138.4 | **P+ better** |
| N vs P+ | q2_p99_us | 215128 | 5438 | 1420 | 331.5 | 3852 | 1129 | -99.3% | 55.5 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 906.1 | 79.2 | 2253 | 96.9 | 88.5 | 227.6 | +148.6% | 15.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 6779 | 151.5 | 7637 | 624.0 | 454.1 | 452.7 | +12.7% | 1.89 | no difference |
| N vs P+ | c4_q2_p50_us | 206457 | 3683 | 1966 | 109.8 | 2606 | 4821 | -99.0% | 78.5 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 436422 | 16854 | 7682 | 696.2 | 11928 | 56236 | -98.2% | 35.9 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 249.4 | 10.2 | 327.0 | 16.7 | 13.8 | 316.9 | +31.1% | 5.61 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 310.5 | 0.33 | 92.2 | 0.32 | 0.32 | 2.11 | -70.3% | 673.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 28519 | 2296 | 4449 | 380.9 | 1645 | 29375 | -84.4% | 14.6 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 775.8 | 79.9 | 505.2 | 24.3 | 59.1 | 148.3 | -34.9% | 4.58 | **H3 better** |
| N vs H3 | q1_p99_us | 1835 | 149.8 | 2336 | 303.9 | 239.6 | 1339 | +27.3% | 2.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 103967 | 1051 | 479.8 | 28.3 | 743.2 | 3206 | -99.5% | 139.2 | **H3 better** |
| N vs H3 | q2_p99_us | 215128 | 5438 | 1821 | 145.4 | 3846 | 1508 | -99.2% | 55.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 906.1 | 79.2 | 1077 | 54.0 | 67.8 | 259.6 | +18.9% | 2.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 6779 | 151.5 | 3983 | 566.7 | 414.8 | 405.7 | -41.2% | 6.74 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 206457 | 3683 | 1074 | 75.5 | 2605 | 4822 | -99.5% | 78.8 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 436422 | 16854 | 3784 | 427.6 | 11921 | 56248 | -99.1% | 36.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | view_state_bytes_per_key_read | 249.4 | 10.2 | 110.4 | 0.86 | 7.21 | 179.0 | -55.7% | 19.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 310.5 | 0.33 | 110.4 | 0.79 | 0.61 | 2.18 | -64.4% | 330.1 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 28519 | 2296 | 5647 | 378.6 | 1645 | 29750 | -80.2% | 13.9 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 836.1 | 105.6 | 505.2 | 24.3 | 76.6 | 109.0 | -39.6% | 4.32 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1910 | 381.8 | 2336 | 303.9 | 345.1 | 1305 | +22.3% | 1.23 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 648.5 | 100.2 | 479.8 | 28.3 | 73.6 | 75.8 | -26.0% | 2.29 | no difference |
| P+ vs H3 | q2_p99_us | 1420 | 331.5 | 1821 | 145.4 | 256.0 | 1011 | +28.2% | 1.57 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2253 | 96.9 | 1077 | 54.0 | 78.5 | 126.9 | -52.2% | 15.0 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 7637 | 624.0 | 3983 | 566.7 | 596.1 | 337.4 | -47.8% | 6.13 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1966 | 109.8 | 1074 | 75.5 | 94.3 | 196.7 | -45.4% | 9.47 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7682 | 696.2 | 3784 | 427.6 | 577.7 | 1842 | -50.7% | 6.75 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | view_state_bytes_per_key_read | 327.0 | 16.7 | 110.4 | 0.86 | 11.9 | 262.3 | -66.2% | 18.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 92.2 | 0.32 | 110.4 | 0.79 | 0.60 | 1.61 | +19.8% | 30.2 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 4449 | 380.9 | 5647 | 378.6 | 379.8 | 4730 | +26.9% | 3.16 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 834.1 | 54.9 | 854.4 | 35.8 | 46.3 | 99.6 | +2.4% | 0.44 | BELOW FLOOR |
| N vs P+ | q1_p99_us | 1796 | 108.7 | 2066 | 281.7 | 213.5 | 306.3 | +15.0% | 1.26 | BELOW FLOOR |
| N vs P+ | q2_p50_us | 107881 | 1753 | 663.9 | 39.6 | 1240 | 14689 | -99.4% | 86.5 | **P+ better** |
| N vs P+ | q2_p99_us | 233899 | 8686 | 1601 | 337.6 | 6146 | 6517 | -99.3% | 37.8 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 911.2 | 54.9 | 2467 | 110.7 | 87.3 | 165.3 | +170.7% | 17.8 | **N better** |
| N vs P+ | c4_q1_p99_us | 6942 | 107.3 | 8825 | 987.8 | 702.6 | 1931 | +27.1% | 2.68 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 209897 | 2564 | 2144 | 113.6 | 1815 | 22873 | -99.0% | 114.5 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 445998 | 10343 | 8140 | 452.5 | 7321 | 5464 | -98.2% | 59.8 | **P+ better** |
| N vs P+ | view_state_bytes_per_key_read | 247.9 | 9.43 | 326.1 | 13.8 | 11.8 | 294.0 | +31.5% | 6.62 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 310.5 | 0.54 | 92.5 | 0.15 | 0.39 | 1.62 | -70.2% | 554.0 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 28390 | 2332 | 4449 | 360.8 | 1669 | 29858 | -84.3% | 14.3 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 834.1 | 54.9 | 566.3 | 45.1 | 50.2 | 122.3 | -32.1% | 5.33 | **H3 better** |
| N vs H3 | q1_p99_us | 1796 | 108.7 | 2195 | 315.6 | 236.0 | 411.4 | +22.2% | 1.69 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 107881 | 1753 | 532.9 | 47.2 | 1240 | 14689 | -99.5% | 86.6 | **H3 better** |
| N vs H3 | q2_p99_us | 233899 | 8686 | 1714 | 216.5 | 6143 | 6560 | -99.3% | 37.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 911.2 | 54.9 | 1161 | 47.9 | 51.5 | 169.0 | +27.4% | 4.85 | **N better** |
| N vs H3 | c4_q1_p99_us | 6942 | 107.3 | 4257 | 473.8 | 343.5 | 682.2 | -38.7% | 7.82 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 209897 | 2564 | 1125 | 70.4 | 1814 | 22873 | -99.5% | 115.1 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 445998 | 10343 | 3794 | 296.5 | 7317 | 5312 | -99.1% | 60.4 | **H3 better** |
| N vs H3 | view_state_bytes_per_key_read | 247.9 | 9.43 | 109.3 | 0.43 | 6.67 | 179.0 | -55.9% | 20.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 310.5 | 0.54 | 110.4 | 0.60 | 0.57 | 2.46 | -64.5% | 353.0 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 28390 | 2332 | 5611 | 354.1 | 1668 | 30245 | -80.2% | 13.7 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 854.4 | 35.8 | 566.3 | 45.1 | 40.7 | 78.1 | -33.7% | 7.08 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2066 | 281.7 | 2195 | 315.6 | 299.2 | 279.7 | +6.2% | 0.43 | BELOW FLOOR |
| P+ vs H3 | q2_p50_us | 663.9 | 39.6 | 532.9 | 47.2 | 43.6 | 80.3 | -19.7% | 3.00 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1601 | 337.6 | 1714 | 216.5 | 283.6 | 780.3 | +7.1% | 0.40 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2467 | 110.7 | 1161 | 47.9 | 85.3 | 43.8 | -52.9% | 15.3 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 8825 | 987.8 | 4257 | 473.8 | 774.7 | 2016 | -51.8% | 5.90 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 2144 | 113.6 | 1125 | 70.4 | 94.5 | 168.1 | -47.5% | 10.8 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8140 | 452.5 | 3794 | 296.5 | 382.6 | 1464 | -53.4% | 11.4 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 326.1 | 13.8 | 109.3 | 0.43 | 9.76 | 233.4 | -66.5% | 22.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 92.5 | 0.15 | 110.4 | 0.60 | 0.44 | 2.37 | +19.3% | 40.9 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 4449 | 360.8 | 5611 | 354.1 | 357.5 | 4855 | +26.1% | 3.25 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1373 | 81.8 | 868.8 | 37.7 | 63.7 | 100.6 | -36.7% | 7.92 | **P+ better** |
| N vs P+ | q1_p99_us | 3141 | 501.4 | 1837 | 265.1 | 401.1 | 393.3 | -41.5% | 3.25 | **P+ better** |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 4857 | 784.7 | 2591 | 104.7 | 559.8 | 55.7 | -46.7% | 4.05 | **P+ better** |
| N vs P+ | c4_q1_p99_us | 15740 | 3045 | 8442 | 631.6 | 2199 | 514.8 | -46.4% | 3.32 | **P+ better** |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | view_state_bytes_per_key_read | 2032 | 444.7 | 1444 | 291.3 | 375.9 | 5811 | -28.9% | 1.56 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 271.0 | 0.38 | 476.6 | 0.04 | 0.27 | 0.49 | +75.9% | 767.3 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 680.4 | 129.7 | 14204 | 3293 | 2330 | 36220 | +1987.7% | 5.80 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 1373 | 81.8 | 1314 | 112.2 | 98.2 | 41.1 | -4.3% | 0.61 | no difference |
| N vs H3 | q1_p99_us | 3141 | 501.4 | 3055 | 475.5 | 488.6 | 411.9 | -2.7% | 0.18 | BELOW FLOOR |
| N vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q1_p50_us | 4857 | 784.7 | 3923 | 417.0 | 628.3 | 11.9 | -19.2% | 1.49 | no difference |
| N vs H3 | c4_q1_p99_us | 15740 | 3045 | 10243 | 889.0 | 2243 | 3437 | -34.9% | 2.45 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | view_state_bytes_per_key_read | 2032 | 444.7 | 141.0 | 9.01 | 314.5 | 4826 | -93.1% | 6.01 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 271.0 | 0.38 | 498.3 | 0.85 | 0.66 | 1.64 | +83.9% | 346.1 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 680.4 | 129.7 | 16000 | 3639 | 2575 | 39948 | +2251.8% | 5.95 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 868.8 | 37.7 | 1314 | 112.2 | 83.7 | 93.5 | +51.2% | 5.31 | **P+ better** |
| P+ vs H3 | q1_p99_us | 1837 | 265.1 | 3055 | 475.5 | 385.0 | 569.3 | +66.3% | 3.16 | **P+ better** |
| P+ vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q1_p50_us | 2591 | 104.7 | 3923 | 417.0 | 304.0 | 56.6 | +51.4% | 4.38 | **P+ better** |
| P+ vs H3 | c4_q1_p99_us | 8442 | 631.6 | 10243 | 889.0 | 771.1 | 3467 | +21.3% | 2.34 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | view_state_bytes_per_key_read | 1444 | 291.3 | 141.0 | 9.01 | 206.1 | 3237 | -90.2% | 6.32 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 476.6 | 0.04 | 498.3 | 0.85 | 0.60 | 1.57 | +4.6% | 36.1 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 14204 | 3293 | 16000 | 3639 | 3470 | 53886 | +12.6% | 0.52 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1321 | 54.2 | 754.6 | 32.5 | 44.7 | 32.8 | -42.9% | 12.7 | **P+ better** |
| N vs P+ | q1_p99_us | 3204 | 311.3 | 1776 | 230.8 | 274.0 | 119.3 | -44.6% | 5.21 | **P+ better** |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 4227 | 491.2 | 2332 | 66.4 | 350.5 | 225.3 | -44.8% | 5.41 | **P+ better** |
| N vs P+ | c4_q1_p99_us | 13645 | 1391 | 7221 | 314.2 | 1008 | 2164 | -47.1% | 6.37 | **P+ better** |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | view_state_bytes_per_key_read | 2036 | 443.6 | 1445 | 285.6 | 373.1 | 5804 | -29.1% | 1.59 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 271.4 | 0.34 | 477.7 | 0.04 | 0.24 | 0.45 | +76.0% | 852.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 697.8 | 139.2 | 14767 | 3384 | 2395 | 37481 | +2016.3% | 5.88 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 1321 | 54.2 | 1174 | 90.3 | 74.5 | 27.0 | -11.1% | 1.97 | no difference |
| N vs H3 | q1_p99_us | 3204 | 311.3 | 2460 | 338.3 | 325.1 | 1711 | -23.2% | 2.29 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q1_p50_us | 4227 | 491.2 | 3366 | 305.7 | 409.1 | 141.3 | -20.4% | 2.11 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 13645 | 1391 | 9302 | 788.8 | 1131 | 1353 | -31.8% | 3.84 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | view_state_bytes_per_key_read | 2036 | 443.6 | 140.4 | 8.78 | 313.7 | 4817 | -93.1% | 6.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 271.4 | 0.34 | 499.7 | 0.85 | 0.64 | 1.62 | +84.1% | 354.4 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 697.8 | 139.2 | 16151 | 3645 | 2580 | 40103 | +2214.6% | 5.99 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 754.6 | 32.5 | 1174 | 90.3 | 67.9 | 42.4 | +55.6% | 6.18 | **P+ better** |
| P+ vs H3 | q1_p99_us | 1776 | 230.8 | 2460 | 338.3 | 289.6 | 1715 | +38.5% | 2.36 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q1_p50_us | 2332 | 66.4 | 3366 | 305.7 | 221.2 | 263.9 | +44.3% | 4.67 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 7221 | 314.2 | 9302 | 788.8 | 600.4 | 2514 | +28.8% | 3.47 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | view_state_bytes_per_key_read | 1445 | 285.6 | 140.4 | 8.78 | 202.0 | 3237 | -90.3% | 6.46 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 477.7 | 0.04 | 499.7 | 0.85 | 0.60 | 1.56 | +4.6% | 36.7 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 14767 | 3384 | 16151 | 3645 | 3517 | 54853 | +9.4% | 0.39 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1299 | 114.6 | 823.9 | 31.7 | 84.1 | 83.3 | -36.6% | 5.66 | **P+ better** |
| N vs P+ | q1_p99_us | 3289 | 751.8 | 1948 | 125.6 | 539.0 | 365.5 | -40.8% | 2.49 | no difference |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 4109 | 355.0 | 2491 | 137.8 | 269.2 | 161.3 | -39.4% | 6.01 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 13379 | 1185 | 8537 | 1231 | 1208 | 1936 | -36.2% | 4.01 | **P+ better** |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | view_state_bytes_per_key_read | 2034 | 443.6 | 1448 | 285.9 | 373.2 | 5817 | -28.8% | 1.57 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 274.9 | 0.32 | 478.1 | 0.06 | 0.23 | 5.86 | +73.9% | 869.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 989.2 | 206.7 | 15045 | 3519 | 2493 | 38095 | +1420.9% | 5.64 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 1299 | 114.6 | 1263 | 61.5 | 91.9 | 51.6 | -2.8% | 0.40 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 3289 | 751.8 | 3206 | 414.4 | 607.1 | 202.0 | -2.5% | 0.14 | BELOW FLOOR |
| N vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q1_p50_us | 4109 | 355.0 | 3732 | 153.5 | 273.5 | 205.7 | -9.2% | 1.38 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 13379 | 1185 | 9973 | 721.8 | 981.0 | 2974 | -25.5% | 3.47 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | view_state_bytes_per_key_read | 2034 | 443.6 | 139.9 | 8.18 | 313.7 | 4819 | -93.1% | 6.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 274.9 | 0.32 | 499.5 | 1.00 | 0.74 | 6.14 | +81.7% | 302.0 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 989.2 | 206.7 | 16133 | 3610 | 2557 | 40348 | +1530.8% | 5.92 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 823.9 | 31.7 | 1263 | 61.5 | 48.9 | 92.4 | +53.3% | 8.97 | **P+ better** |
| P+ vs H3 | q1_p99_us | 1948 | 125.6 | 3206 | 414.4 | 306.2 | 414.7 | +64.6% | 4.11 | **P+ better** |
| P+ vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q1_p50_us | 2491 | 137.8 | 3732 | 153.5 | 145.9 | 128.3 | +49.8% | 8.51 | **P+ better** |
| P+ vs H3 | c4_q1_p99_us | 8537 | 1231 | 9973 | 721.8 | 1009 | 3538 | +16.8% | 1.42 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | view_state_bytes_per_key_read | 1448 | 285.9 | 139.9 | 8.18 | 202.2 | 3258 | -90.3% | 6.47 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 478.1 | 0.06 | 499.5 | 1.00 | 0.71 | 1.84 | +4.5% | 30.2 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 15045 | 3519 | 16133 | 3610 | 3565 | 55455 | +7.2% | 0.31 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1319 | 106.2 | 810.9 | 23.8 | 77.0 | 34.8 | -38.5% | 6.61 | **P+ better** |
| N vs P+ | q1_p99_us | 3057 | 907.7 | 1925 | 381.5 | 696.2 | 115.7 | -37.0% | 1.63 | no difference |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 4210 | 641.8 | 2391 | 99.1 | 459.2 | 201.9 | -43.2% | 3.96 | **P+ better** |
| N vs P+ | c4_q1_p99_us | 13202 | 1969 | 8536 | 1585 | 1787 | 1076 | -35.3% | 2.61 | no difference |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | view_state_bytes_per_key_read | 2040 | 451.9 | 1452 | 295.2 | 381.7 | 5806 | -28.8% | 1.54 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 271.4 | 0.36 | 477.5 | 0.07 | 0.26 | 0.98 | +75.9% | 802.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 712.3 | 133.6 | 14648 | 3425 | 2424 | 37081 | +1956.4% | 5.75 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H3 | q1_p50_us | 1319 | 106.2 | 1202 | 84.3 | 95.9 | 21.6 | -8.9% | 1.22 | no difference |
| N vs H3 | q1_p99_us | 3057 | 907.7 | 2934 | 792.8 | 852.2 | 1018 | -4.0% | 0.14 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q1_p50_us | 4210 | 641.8 | 3384 | 315.9 | 505.8 | 8.11 | -19.6% | 1.63 | no difference |
| N vs H3 | c4_q1_p99_us | 13202 | 1969 | 8794 | 658.6 | 1468 | 985.3 | -33.4% | 3.00 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | view_state_bytes_per_key_read | 2040 | 451.9 | 141.3 | 9.33 | 319.6 | 4806 | -93.1% | 5.94 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 271.4 | 0.36 | 499.6 | 1.04 | 0.78 | 1.94 | +84.1% | 293.8 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 712.3 | 133.6 | 16313 | 3815 | 2699 | 40405 | +2190.2% | 5.78 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 810.9 | 23.8 | 1202 | 84.3 | 61.9 | 40.9 | +48.2% | 6.31 | **P+ better** |
| P+ vs H3 | q1_p99_us | 1925 | 381.5 | 2934 | 792.8 | 622.1 | 1014 | +52.4% | 1.62 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q1_p50_us | 2391 | 99.1 | 3384 | 315.9 | 234.1 | 201.8 | +41.5% | 4.24 | **P+ better** |
| P+ vs H3 | c4_q1_p99_us | 8536 | 1585 | 8794 | 658.6 | 1214 | 1407 | +3.0% | 0.21 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | view_state_bytes_per_key_read | 1452 | 295.2 | 141.3 | 9.33 | 208.9 | 3258 | -90.3% | 6.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 477.5 | 0.07 | 499.6 | 1.04 | 0.74 | 1.67 | +4.6% | 30.0 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 14648 | 3425 | 16313 | 3815 | 3625 | 54833 | +11.4% | 0.46 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1487 | 139.3 | 820.3 | 62.8 | 108.0 | 70.5 | -44.8% | 6.17 | **P+ better** |
| N vs P+ | q1_p99_us | 3957 | 447.1 | 2015 | 401.7 | 425.0 | 2304 | -49.1% | 4.57 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 5397 | 766.6 | 2719 | 177.5 | 556.4 | 112.7 | -49.6% | 4.81 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 19374 | 2319 | 8468 | 537.5 | 1683 | 3843 | -56.3% | 6.48 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | view_state_bytes_per_key_read | 2032 | 443.9 | 1446 | 295.8 | 377.2 | 5910 | -28.8% | 1.55 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 271.2 | 0.31 | 476.0 | 0.05 | 0.23 | 0.91 | +75.5% | 908.7 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 696.0 | 139.0 | 14713 | 3399 | 2405 | 38162 | +2013.9% | 5.83 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 1487 | 139.3 | 1340 | 73.2 | 111.2 | 54.5 | -9.9% | 1.33 | no difference |
| N vs H3 | q1_p99_us | 3957 | 447.1 | 4086 | 109.8 | 325.6 | 1259 | +3.3% | 0.40 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q1_p50_us | 5397 | 766.6 | 4030 | 475.7 | 638.0 | 113.9 | -25.3% | 2.14 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 19374 | 2319 | 12244 | 1799 | 2075 | 4203 | -36.8% | 3.44 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | view_state_bytes_per_key_read | 2032 | 443.9 | 140.6 | 8.43 | 314.0 | 4900 | -93.1% | 6.02 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 271.2 | 0.31 | 497.2 | 1.23 | 0.90 | 1.99 | +83.3% | 250.9 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 696.0 | 139.0 | 16148 | 3668 | 2595 | 40910 | +2220.1% | 5.95 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 820.3 | 62.8 | 1340 | 73.2 | 68.2 | 60.8 | +63.3% | 7.62 | **P+ better** |
| P+ vs H3 | q1_p99_us | 2015 | 401.7 | 4086 | 109.8 | 294.5 | 2613 | +102.8% | 7.03 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q1_p50_us | 2719 | 177.5 | 4030 | 475.7 | 359.0 | 17.2 | +48.2% | 3.65 | **P+ better** |
| P+ vs H3 | c4_q1_p99_us | 8468 | 537.5 | 12244 | 1799 | 1328 | 3623 | +44.6% | 2.84 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs H3 | view_state_bytes_per_key_read | 1446 | 295.8 | 140.6 | 8.43 | 209.2 | 3304 | -90.3% | 6.24 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 476.0 | 0.05 | 497.2 | 1.23 | 0.87 | 1.81 | +4.5% | 24.3 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 14713 | 3399 | 16148 | 3668 | 3536 | 55914 | +9.8% | 0.41 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 149.9 | 5.15 | 722.1 | 19.3 | 14.2 | 39.2 | +381.7% | 40.4 | **N better** |
| N vs P+ | q1_p99_us | 351.3 | 73.0 | 1632 | 298.9 | 217.6 | 423.8 | +364.6% | 5.89 | **N better** |
| N vs P+ | q2_p50_us | 153.0 | 3.56 | 536.4 | 13.4 | 9.77 | 78.3 | +250.7% | 39.2 | **N better** |
| N vs P+ | q2_p99_us | 592.7 | 49.3 | 1275 | 150.8 | 112.2 | 234.4 | +115.2% | 6.09 | **N better** |
| N vs P+ | c4_q1_p50_us | 213.1 | 6.43 | 2148 | 72.3 | 51.3 | 11.6 | +908.2% | 37.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 941.6 | 217.9 | 10406 | 3825 | 2709 | 250610 | +1005.1% | 3.49 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 226.9 | 5.13 | 1814 | 35.8 | 25.5 | 258.8 | +699.8% | 62.1 | **N better** |
| N vs P+ | c4_q2_p99_us | 886.3 | 121.4 | 6463 | 636.2 | 458.0 | 725.0 | +629.2% | 12.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 325.1 | 0.02 | 1495 | 233.5 | 165.1 | 289.6 | +359.9% | 7.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | pss_mib | 12.8 | 0.00 | 45.6 | 0.00 | 0.00 | 1.10 | +256.0% | 7777 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 7310 | 5.12 | -6266 | 3.58 | 4.42 | 284.5 | -185.7% | 3072 | **P+ better** |
| N vs H3 | q1_p50_us | 149.9 | 5.15 | 300.5 | 25.2 | 18.2 | 33.0 | +100.4% | 8.28 | **N better** |
| N vs H3 | q1_p99_us | 351.3 | 73.0 | 799.2 | 127.6 | 103.9 | 51.3 | +127.5% | 4.31 | **N better** |
| N vs H3 | q2_p50_us | 153.0 | 3.56 | 286.8 | 24.8 | 17.7 | 12.0 | +87.5% | 7.56 | **N better** |
| N vs H3 | q2_p99_us | 592.7 | 49.3 | 830.3 | 157.3 | 116.6 | 192.2 | +40.1% | 2.04 | no difference |
| N vs H3 | c4_q1_p50_us | 213.1 | 6.43 | 536.2 | 23.4 | 17.2 | 23.6 | +151.6% | 18.8 | **N better** |
| N vs H3 | c4_q1_p99_us | 941.6 | 217.9 | 1943 | 222.5 | 220.2 | 1266 | +106.4% | 4.55 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 226.9 | 5.13 | 525.8 | 22.1 | 16.0 | 26.8 | +131.7% | 18.6 | **N better** |
| N vs H3 | c4_q2_p99_us | 886.3 | 121.4 | 1860 | 180.7 | 154.0 | 555.5 | +109.8% | 6.32 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 325.1 | 0.02 | 108.0 | 0.26 | 0.18 | 7.18 | -66.8% | 1176 | **H3 better** |
| N vs H3 | pss_mib | 12.8 | 0.00 | 72.9 | 0.09 | 0.06 | 0.44 | +469.6% | 945.0 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 7310 | 5.12 | 15696 | 94.2 | 66.7 | 559.5 | +114.7% | 125.7 | **N better** |
| P+ vs H3 | q1_p50_us | 722.1 | 19.3 | 300.5 | 25.2 | 22.5 | 33.0 | -58.4% | 18.8 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1632 | 298.9 | 799.2 | 127.6 | 229.8 | 426.8 | -51.0% | 3.62 | **H3 better** |
| P+ vs H3 | q2_p50_us | 536.4 | 13.4 | 286.8 | 24.8 | 19.9 | 78.8 | -46.5% | 12.6 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1275 | 150.8 | 830.3 | 157.3 | 154.1 | 146.6 | -34.9% | 2.89 | no difference |
| P+ vs H3 | c4_q1_p50_us | 2148 | 72.3 | 536.2 | 23.4 | 53.7 | 21.6 | -75.0% | 30.0 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 10406 | 3825 | 1943 | 222.5 | 2709 | 250614 | -81.3% | 3.12 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1814 | 35.8 | 525.8 | 22.1 | 29.7 | 260.1 | -71.0% | 43.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6463 | 636.2 | 1860 | 180.7 | 467.6 | 525.2 | -71.2% | 9.84 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 1495 | 233.5 | 108.0 | 0.26 | 165.1 | 289.5 | -92.8% | 8.40 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.6 | 0.00 | 72.9 | 0.09 | 0.06 | 1.01 | +60.0% | 430.1 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -6266 | 3.58 | 15696 | 94.2 | 66.7 | 504.7 | -350.5% | 329.4 | **P+ better** |

## `single`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 145.2 | 1.87 | 695.3 | 18.7 | 13.3 | 20.8 | +378.9% | 41.4 | **N better** |
| N vs P+ | q1_p99_us | 265.5 | 27.4 | 1459 | 133.3 | 96.2 | 373.1 | +449.4% | 12.4 | **N better** |
| N vs P+ | q2_p50_us | 150.9 | 3.03 | 519.0 | 10.2 | 7.51 | 16.0 | +243.9% | 49.0 | **N better** |
| N vs P+ | q2_p99_us | 538.5 | 22.3 | 1059 | 135.5 | 97.1 | 296.3 | +96.7% | 5.36 | **N better** |
| N vs P+ | c4_q1_p50_us | 216.4 | 3.95 | 2013 | 44.1 | 31.3 | 135.9 | +830.3% | 57.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 717.3 | 86.1 | 12248 | 7062 | 4994 | 379.4 | +1607.6% | 2.31 | no difference |
| N vs P+ | c4_q2_p50_us | 244.6 | 10.9 | 1667 | 16.7 | 14.1 | 130.9 | +581.7% | 101.0 | **N better** |
| N vs P+ | c4_q2_p99_us | 844.9 | 52.6 | 5549 | 244.5 | 176.9 | 491.3 | +556.8% | 26.6 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 320.4 | 0.01 | 1368 | 20.5 | 14.5 | 332.3 | +327.0% | 72.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | pss_mib | 12.9 | 0.08 | 45.6 | 0.01 | 0.06 | 0.10 | +252.5% | 586.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 7513 | 81.9 | -2548 | 10.8 | 58.4 | 145.2 | -133.9% | 172.2 | **P+ better** |
| N vs H3 | q1_p50_us | 145.2 | 1.87 | 290.6 | 10.5 | 7.51 | 57.5 | +100.1% | 19.4 | **N better** |
| N vs H3 | q1_p99_us | 265.5 | 27.4 | 666.5 | 43.5 | 36.4 | 358.2 | +151.1% | 11.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 150.9 | 3.03 | 275.7 | 11.3 | 8.25 | 41.6 | +82.7% | 15.1 | **N better** |
| N vs H3 | q2_p99_us | 538.5 | 22.3 | 620.1 | 58.3 | 44.1 | 379.6 | +15.1% | 1.85 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 216.4 | 3.95 | 550.8 | 40.6 | 28.8 | 12.7 | +154.6% | 11.6 | **N better** |
| N vs H3 | c4_q1_p99_us | 717.3 | 86.1 | 1905 | 156.0 | 126.0 | 427.7 | +165.7% | 9.43 | **N better** |
| N vs H3 | c4_q2_p50_us | 244.6 | 10.9 | 532.7 | 34.0 | 25.2 | 29.7 | +117.8% | 11.4 | **N better** |
| N vs H3 | c4_q2_p99_us | 844.9 | 52.6 | 1764 | 138.7 | 104.9 | 267.8 | +108.8% | 8.76 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 320.4 | 0.01 | 110.3 | 0.10 | 0.07 | 3.64 | -65.6% | 2848 | **H3 better** |
| N vs H3 | pss_mib | 12.9 | 0.08 | 72.4 | 0.19 | 0.14 | 0.05 | +459.9% | 411.4 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 7513 | 81.9 | 15290 | 198.1 | 151.6 | 100.5 | +103.5% | 51.3 | **N better** |
| P+ vs H3 | q1_p50_us | 695.3 | 18.7 | 290.6 | 10.5 | 15.1 | 61.1 | -58.2% | 26.7 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1459 | 133.3 | 666.5 | 43.5 | 99.1 | 515.3 | -54.3% | 7.99 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 519.0 | 10.2 | 275.7 | 11.3 | 10.7 | 43.1 | -46.9% | 22.7 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1059 | 135.5 | 620.1 | 58.3 | 104.3 | 469.7 | -41.5% | 4.21 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2013 | 44.1 | 550.8 | 40.6 | 42.4 | 135.8 | -72.6% | 34.5 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 12248 | 7062 | 1905 | 156.0 | 4995 | 549.7 | -84.4% | 2.07 | no difference |
| P+ vs H3 | c4_q2_p50_us | 1667 | 16.7 | 532.7 | 34.0 | 26.8 | 133.9 | -68.0% | 42.4 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5549 | 244.5 | 1764 | 138.7 | 198.8 | 512.1 | -68.2% | 19.0 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 1368 | 20.5 | 110.3 | 0.10 | 14.5 | 332.3 | -91.9% | 86.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.6 | 0.01 | 72.4 | 0.19 | 0.13 | 0.11 | +58.9% | 200.5 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2548 | 10.8 | 15290 | 198.1 | 140.3 | 153.8 | -700.1% | 127.1 | **P+ better** |

## `single`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 148.1 | 5.45 | 715.8 | 39.8 | 28.4 | 80.8 | +383.4% | 20.0 | **N better** |
| N vs P+ | q1_p99_us | 289.2 | 62.2 | 1512 | 110.4 | 89.6 | 146.5 | +422.8% | 13.6 | **N better** |
| N vs P+ | q2_p50_us | 155.4 | 8.34 | 542.2 | 31.2 | 22.8 | 22.4 | +248.8% | 16.9 | **N better** |
| N vs P+ | q2_p99_us | 536.5 | 63.2 | 1079 | 92.1 | 79.0 | 194.6 | +101.1% | 6.86 | **N better** |
| N vs P+ | c4_q1_p50_us | 217.4 | 5.74 | 2070 | 72.8 | 51.7 | 6.13 | +852.5% | 35.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 704.8 | 114.5 | 8946 | 3819 | 2702 | 9096 | +1169.3% | 3.05 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 226.3 | 3.27 | 1723 | 78.4 | 55.5 | 46.5 | +661.3% | 27.0 | **N better** |
| N vs P+ | c4_q2_p99_us | 775.7 | 49.5 | 6325 | 491.7 | 349.5 | 1220 | +715.4% | 15.9 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 323.3 | 0.00 | 1380 | 348.2 | 246.2 | 284.4 | +327.0% | 4.29 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | pss_mib | 12.7 | 0.06 | 45.8 | 0.02 | 0.04 | 0.91 | +260.1% | 751.3 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 7334 | 61.4 | -2506 | 22.0 | 46.1 | 701.6 | -134.2% | 213.2 | **P+ better** |
| N vs H3 | q1_p50_us | 148.1 | 5.45 | 287.5 | 16.5 | 12.3 | 28.4 | +94.2% | 11.3 | **N better** |
| N vs H3 | q1_p99_us | 289.2 | 62.2 | 760.9 | 88.2 | 76.4 | 173.6 | +163.1% | 6.18 | **N better** |
| N vs H3 | q2_p50_us | 155.4 | 8.34 | 273.9 | 15.5 | 12.5 | 12.3 | +76.2% | 9.50 | **N better** |
| N vs H3 | q2_p99_us | 536.5 | 63.2 | 629.4 | 49.4 | 56.7 | 159.0 | +17.3% | 1.64 | BELOW FLOOR |
| N vs H3 | c4_q1_p50_us | 217.4 | 5.74 | 531.1 | 14.4 | 11.0 | 95.4 | +144.3% | 28.5 | **N better** |
| N vs H3 | c4_q1_p99_us | 704.8 | 114.5 | 2194 | 198.3 | 161.9 | 116.2 | +211.2% | 9.20 | **N better** |
| N vs H3 | c4_q2_p50_us | 226.3 | 3.27 | 517.5 | 14.5 | 10.5 | 85.2 | +128.7% | 27.7 | **N better** |
| N vs H3 | c4_q2_p99_us | 775.7 | 49.5 | 2041 | 326.6 | 233.6 | 303.5 | +163.1% | 5.42 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 323.3 | 0.00 | 112.4 | 0.09 | 0.06 | 1.87 | -65.2% | 3388 | **H3 better** |
| N vs H3 | pss_mib | 12.7 | 0.06 | 72.6 | 0.06 | 0.06 | 0.30 | +470.8% | 1043 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 7334 | 61.4 | 15534 | 58.9 | 60.2 | 899.6 | +111.8% | 136.3 | **N better** |
| P+ vs H3 | q1_p50_us | 715.8 | 39.8 | 287.5 | 16.5 | 30.5 | 85.0 | -59.8% | 14.1 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1512 | 110.4 | 760.9 | 88.2 | 99.9 | 204.6 | -49.7% | 7.52 | **H3 better** |
| P+ vs H3 | q2_p50_us | 542.2 | 31.2 | 273.9 | 15.5 | 24.6 | 21.3 | -49.5% | 10.9 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1079 | 92.1 | 629.4 | 49.4 | 73.9 | 248.5 | -41.7% | 6.08 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2070 | 72.8 | 531.1 | 14.4 | 52.5 | 95.6 | -74.3% | 29.3 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 8946 | 3819 | 2194 | 198.3 | 2704 | 9095 | -75.5% | 2.50 | BELOW FLOOR |
| P+ vs H3 | c4_q2_p50_us | 1723 | 78.4 | 517.5 | 14.5 | 56.4 | 85.7 | -70.0% | 21.4 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6325 | 491.7 | 2041 | 326.6 | 417.4 | 1240 | -67.7% | 10.3 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 1380 | 348.2 | 112.4 | 0.09 | 246.2 | 284.4 | -91.9% | 5.15 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.8 | 0.02 | 72.6 | 0.06 | 0.04 | 0.96 | +58.5% | 632.0 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2506 | 22.0 | 15534 | 58.9 | 44.4 | 1070 | -719.9% | 405.8 | **P+ better** |

## `single`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 152.9 | 5.09 | 713.3 | 23.3 | 16.9 | 82.1 | +366.4% | 33.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 323.5 | 19.8 | 1358 | 117.5 | 84.2 | 509.8 | +319.9% | 12.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 161.8 | 7.43 | 523.7 | 21.3 | 16.0 | 58.2 | +223.7% | 22.7 | **N better** |
| N vs P+ | q2_p99_us | 617.9 | 49.2 | 1006 | 124.5 | 94.7 | 252.3 | +62.9% | 4.10 | **N better** |
| N vs P+ | c4_q1_p50_us | 212.2 | 5.45 | 2114 | 94.2 | 66.7 | 52.5 | +896.2% | 28.5 | **N better** |
| N vs P+ | c4_q1_p99_us | 705.8 | 47.2 | 13157 | 6789 | 4801 | 13519 | +1763.9% | 2.59 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 230.8 | 6.17 | 1804 | 103.4 | 73.3 | 46.8 | +681.6% | 21.5 | **N better** |
| N vs P+ | c4_q2_p99_us | 833.5 | 28.9 | 6404 | 645.3 | 456.8 | 102.1 | +668.3% | 12.2 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 323.7 | 0.00 | 1286 | 147.5 | 104.3 | 250.3 | +297.4% | 9.23 | **N better** |
| N vs P+ | pss_mib | 12.6 | 0.00 | 45.7 | 0.01 | 0.01 | 0.22 | +263.6% | 3837 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 7165 | 0.00 | -2525 | 12.8 | 9.05 | 253.3 | -135.2% | 1071 | **P+ better** |
| N vs H3 | q1_p50_us | 152.9 | 5.09 | 288.3 | 8.91 | 7.26 | 82.1 | +88.5% | 18.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 323.5 | 19.8 | 668.5 | 62.3 | 46.2 | 87.1 | +106.7% | 7.46 | **N better** |
| N vs H3 | q2_p50_us | 161.8 | 7.43 | 277.5 | 9.95 | 8.78 | 58.0 | +71.6% | 13.2 | **N better** |
| N vs H3 | q2_p99_us | 617.9 | 49.2 | 624.1 | 70.3 | 60.7 | 156.9 | +1.0% | 0.10 | BELOW FLOOR |
| N vs H3 | c4_q1_p50_us | 212.2 | 5.45 | 551.0 | 30.6 | 22.0 | 46.5 | +159.7% | 15.4 | **N better** |
| N vs H3 | c4_q1_p99_us | 705.8 | 47.2 | 1716 | 106.7 | 82.5 | 485.6 | +143.1% | 12.2 | **N better** |
| N vs H3 | c4_q2_p50_us | 230.8 | 6.17 | 542.4 | 30.5 | 22.0 | 77.1 | +135.0% | 14.1 | **N better** |
| N vs H3 | c4_q2_p99_us | 833.5 | 28.9 | 1598 | 108.1 | 79.1 | 77.0 | +91.7% | 9.66 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 323.7 | 0.00 | 109.0 | 0.23 | 0.16 | 1.74 | -66.3% | 1331 | **H3 better** |
| N vs H3 | pss_mib | 12.6 | 0.00 | 72.1 | 0.20 | 0.14 | 0.03 | +474.1% | 430.2 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 7165 | 0.00 | 14915 | 205.3 | 145.2 | 1330 | +108.2% | 53.4 | **N better** |
| P+ vs H3 | q1_p50_us | 713.3 | 23.3 | 288.3 | 8.91 | 17.7 | 86.7 | -59.6% | 24.1 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1358 | 117.5 | 668.5 | 62.3 | 94.0 | 511.2 | -50.8% | 7.34 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 523.7 | 21.3 | 277.5 | 9.95 | 16.6 | 64.2 | -47.0% | 14.8 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1006 | 124.5 | 624.1 | 70.3 | 101.1 | 225.5 | -38.0% | 3.78 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2114 | 94.2 | 551.0 | 30.6 | 70.0 | 53.5 | -73.9% | 22.3 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 13157 | 6789 | 1716 | 106.7 | 4801 | 13526 | -87.0% | 2.38 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1804 | 103.4 | 542.4 | 30.5 | 76.2 | 62.9 | -69.9% | 16.5 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6404 | 645.3 | 1598 | 108.1 | 462.7 | 127.7 | -75.0% | 10.4 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 1286 | 147.5 | 109.0 | 0.23 | 104.3 | 250.3 | -91.5% | 11.3 | **H3 better** |
| P+ vs H3 | pss_mib | 45.7 | 0.01 | 72.1 | 0.20 | 0.14 | 0.22 | +57.9% | 190.6 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2525 | 12.8 | 14915 | 205.3 | 145.5 | 1307 | -690.6% | 119.9 | **P+ better** |

## `single`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 149.6 | 5.08 | 687.7 | 11.7 | 9.03 | 61.2 | +359.6% | 59.6 | **N better** |
| N vs P+ | q1_p99_us | 316.1 | 46.2 | 1328 | 142.1 | 105.6 | 577.3 | +319.9% | 9.58 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 158.3 | 3.90 | 521.4 | 17.5 | 12.7 | 7.92 | +229.4% | 28.6 | **N better** |
| N vs P+ | q2_p99_us | 562.3 | 35.3 | 1052 | 109.9 | 81.6 | 310.4 | +87.1% | 6.00 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 213.2 | 7.58 | 2056 | 133.3 | 94.4 | 9.68 | +864.5% | 19.5 | **N better** |
| N vs P+ | c4_q1_p99_us | 712.7 | 66.5 | 8341 | 2284 | 1616 | 3938 | +1070.3% | 4.72 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 228.0 | 8.30 | 1735 | 106.1 | 75.3 | 12.7 | +660.6% | 20.0 | **N better** |
| N vs P+ | c4_q2_p99_us | 780.4 | 40.9 | 5683 | 440.4 | 312.7 | 506.5 | +628.2% | 15.7 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 321.2 | 0.24 | 1339 | 16.4 | 11.6 | 235.2 | +317.0% | 87.9 | **N better** |
| N vs P+ | pss_mib | 12.3 | 0.01 | 45.5 | 0.03 | 0.02 | 0.30 | +271.4% | 1372 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 6888 | 6.66 | -2621 | 35.3 | 25.4 | 395.5 | -138.1% | 374.1 | **P+ better** |
| N vs H3 | q1_p50_us | 149.6 | 5.08 | 290.2 | 9.99 | 7.93 | 35.3 | +94.0% | 17.7 | **N better** |
| N vs H3 | q1_p99_us | 316.1 | 46.2 | 763.7 | 88.9 | 70.9 | 157.2 | +141.6% | 6.32 | **N better** |
| N vs H3 | q2_p50_us | 158.3 | 3.90 | 276.8 | 9.41 | 7.20 | 39.9 | +74.8% | 16.5 | **N better** |
| N vs H3 | q2_p99_us | 562.3 | 35.3 | 681.0 | 53.8 | 45.5 | 198.0 | +21.1% | 2.61 | BELOW FLOOR |
| N vs H3 | c4_q1_p50_us | 213.2 | 7.58 | 496.0 | 14.6 | 11.6 | 11.0 | +132.7% | 24.4 | **N better** |
| N vs H3 | c4_q1_p99_us | 712.7 | 66.5 | 1901 | 395.5 | 283.6 | 147.6 | +166.7% | 4.19 | **N better** |
| N vs H3 | c4_q2_p50_us | 228.0 | 8.30 | 493.1 | 19.6 | 15.1 | 7.24 | +116.3% | 17.6 | **N better** |
| N vs H3 | c4_q2_p99_us | 780.4 | 40.9 | 1812 | 167.9 | 122.2 | 357.5 | +132.2% | 8.44 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 321.2 | 0.24 | 108.5 | 0.18 | 0.21 | 2.24 | -66.2% | 992.4 | **H3 better** |
| N vs H3 | pss_mib | 12.3 | 0.01 | 72.4 | 0.09 | 0.06 | 0.14 | +490.9% | 934.2 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 6888 | 6.66 | 15404 | 95.2 | 67.5 | 512.0 | +123.6% | 126.2 | **N better** |
| P+ vs H3 | q1_p50_us | 687.7 | 11.7 | 290.2 | 9.99 | 10.9 | 70.2 | -57.8% | 36.5 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1328 | 142.1 | 763.7 | 88.9 | 118.5 | 598.1 | -42.5% | 4.76 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 521.4 | 17.5 | 276.8 | 9.41 | 14.1 | 39.1 | -46.9% | 17.4 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1052 | 109.9 | 681.0 | 53.8 | 86.5 | 335.8 | -35.3% | 4.29 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 2056 | 133.3 | 496.0 | 14.6 | 94.8 | 7.26 | -75.9% | 16.5 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 8341 | 2284 | 1901 | 395.5 | 1639 | 3939 | -77.2% | 3.93 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1735 | 106.1 | 493.1 | 19.6 | 76.3 | 13.0 | -71.6% | 16.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5683 | 440.4 | 1812 | 167.9 | 333.3 | 559.3 | -68.1% | 11.6 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 1339 | 16.4 | 108.5 | 0.18 | 11.6 | 235.2 | -91.9% | 106.2 | **H3 better** |
| P+ vs H3 | pss_mib | 45.5 | 0.03 | 72.4 | 0.09 | 0.07 | 0.28 | +59.1% | 392.6 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2621 | 35.3 | 15404 | 95.2 | 71.8 | 359.4 | -687.6% | 251.0 | **P+ better** |

## `single`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 302.0 | 12.9 | 741.4 | 58.4 | 42.3 | 61.7 | +145.5% | 10.4 | **N better** |
| N vs P+ | q1_p99_us | 536.6 | 40.8 | 1487 | 185.5 | 134.3 | 125.2 | +177.0% | 7.07 | **N better** |
| N vs P+ | q2_p50_us | 182.8 | 6.23 | 529.8 | 21.5 | 15.8 | 23.9 | +189.8% | 21.9 | **N better** |
| N vs P+ | q2_p99_us | 1138 | 170.0 | 1066 | 147.8 | 159.3 | 162.3 | -6.4% | 0.45 | BELOW FLOOR |
| N vs P+ | c4_q1_p50_us | 470.5 | 21.9 | 2079 | 173.9 | 124.0 | 48.0 | +341.9% | 13.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 1521 | 155.8 | 6230 | 1084 | 774.6 | 873.3 | +309.6% | 6.08 | **N better** |
| N vs P+ | c4_q2_p50_us | 284.5 | 9.37 | 1767 | 130.5 | 92.5 | 157.8 | +521.2% | 16.0 | **N better** |
| N vs P+ | c4_q2_p99_us | 1748 | 181.8 | 5887 | 863.5 | 623.9 | 884.0 | +236.7% | 6.63 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 303.4 | 8.48 | 430.2 | 22.1 | 16.7 | 282.0 | +41.8% | 7.58 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 38.3 | 0.26 | 45.8 | 0.02 | 0.19 | 1.18 | +19.6% | 40.5 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 1193 | 63.3 | -269.7 | 15.5 | 46.1 | 475.0 | -122.6% | 31.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H3 | q1_p50_us | 302.0 | 12.9 | 458.5 | 31.0 | 23.8 | 96.1 | +51.8% | 6.58 | **N better** |
| N vs H3 | q1_p99_us | 536.6 | 40.8 | 2141 | 406.8 | 289.1 | 204.8 | +299.0% | 5.55 | **N better** |
| N vs H3 | q2_p50_us | 182.8 | 6.23 | 428.5 | 29.2 | 21.1 | 67.0 | +134.4% | 11.6 | **N better** |
| N vs H3 | q2_p99_us | 1138 | 170.0 | 1695 | 149.3 | 160.0 | 231.2 | +48.9% | 3.48 | **N better** |
| N vs H3 | c4_q1_p50_us | 470.5 | 21.9 | 954.8 | 25.9 | 24.0 | 288.6 | +102.9% | 20.2 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 1521 | 155.8 | 3380 | 177.9 | 167.2 | 222.1 | +122.2% | 11.1 | **N better** |
| N vs H3 | c4_q2_p50_us | 284.5 | 9.37 | 919.2 | 36.7 | 26.8 | 178.9 | +223.1% | 23.7 | **N better** |
| N vs H3 | c4_q2_p99_us | 1748 | 181.8 | 3420 | 345.3 | 276.0 | 515.8 | +95.6% | 6.06 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 303.4 | 8.48 | 109.1 | 0.49 | 6.01 | 185.3 | -64.0% | 32.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 38.3 | 0.26 | 108.3 | 0.13 | 0.21 | 0.68 | +182.9% | 339.7 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 1193 | 63.3 | 6393 | 368.8 | 264.6 | 5314 | +436.0% | 19.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 741.4 | 58.4 | 458.5 | 31.0 | 46.8 | 103.6 | -38.2% | 6.05 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1487 | 185.5 | 2141 | 406.8 | 316.1 | 238.8 | +44.0% | 2.07 | no difference |
| P+ vs H3 | q2_p50_us | 529.8 | 21.5 | 428.5 | 29.2 | 25.6 | 70.4 | -19.1% | 3.95 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1066 | 147.8 | 1695 | 149.3 | 148.6 | 276.2 | +59.0% | 4.23 | **P+ better** |
| P+ vs H3 | c4_q1_p50_us | 2079 | 173.9 | 954.8 | 25.9 | 124.4 | 290.5 | -54.1% | 9.04 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 6230 | 1084 | 3380 | 177.9 | 777.0 | 900.4 | -45.8% | 3.67 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1767 | 130.5 | 919.2 | 36.7 | 95.9 | 238.5 | -48.0% | 8.85 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5887 | 863.5 | 3420 | 345.3 | 657.6 | 729.5 | -41.9% | 3.75 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 430.2 | 22.1 | 109.1 | 0.49 | 15.6 | 212.6 | -74.6% | 20.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.8 | 0.02 | 108.3 | 0.13 | 0.09 | 1.36 | +136.5% | 683.7 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -269.7 | 15.5 | 6393 | 368.8 | 261.0 | 5311 | -2470.5% | 25.5 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 308.9 | 14.1 | 765.2 | 68.8 | 49.6 | 105.6 | +147.7% | 9.19 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 633.0 | 72.5 | 1725 | 258.7 | 190.0 | 142.4 | +172.5% | 5.75 | **N better** |
| N vs P+ | q2_p50_us | 204.0 | 18.7 | 529.9 | 19.6 | 19.2 | 38.1 | +159.7% | 17.0 | **N better** |
| N vs P+ | q2_p99_us | 1384 | 130.2 | 1183 | 140.0 | 135.2 | 416.2 | -14.5% | 1.49 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 496.4 | 35.1 | 2140 | 161.5 | 116.9 | 51.6 | +331.1% | 14.1 | **N better** |
| N vs P+ | c4_q1_p99_us | 1775 | 251.4 | 6286 | 613.1 | 468.6 | 920.0 | +254.2% | 9.63 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 300.3 | 13.2 | 1857 | 117.7 | 83.7 | 72.7 | +518.6% | 18.6 | **N better** |
| N vs P+ | c4_q2_p99_us | 1872 | 262.1 | 6530 | 1124 | 815.8 | 251.1 | +248.7% | 5.71 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 302.8 | 7.92 | 420.0 | 18.7 | 14.4 | 317.2 | +38.7% | 8.15 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 38.1 | 0.55 | 45.8 | 0.01 | 0.39 | 0.46 | +20.1% | 19.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 1172 | 39.4 | -255.2 | 14.0 | 29.6 | 870.6 | -121.8% | 48.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 308.9 | 14.1 | 435.3 | 20.3 | 17.5 | 95.2 | +40.9% | 7.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 633.0 | 72.5 | 2183 | 303.7 | 220.8 | 115.2 | +244.9% | 7.02 | **N better** |
| N vs H3 | q2_p50_us | 204.0 | 18.7 | 410.0 | 15.9 | 17.4 | 47.6 | +100.9% | 11.9 | **N better** |
| N vs H3 | q2_p99_us | 1384 | 130.2 | 1672 | 288.4 | 223.7 | 351.4 | +20.8% | 1.29 | BELOW FLOOR |
| N vs H3 | c4_q1_p50_us | 496.4 | 35.1 | 999.3 | 60.9 | 49.7 | 264.2 | +101.3% | 10.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 1775 | 251.4 | 3531 | 170.6 | 214.8 | 567.7 | +99.0% | 8.18 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 300.3 | 13.2 | 1009 | 55.3 | 40.2 | 264.6 | +236.1% | 17.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 1872 | 262.1 | 3217 | 395.8 | 335.7 | 280.5 | +71.8% | 4.01 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 302.8 | 7.92 | 109.9 | 0.91 | 5.64 | 202.5 | -63.7% | 34.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 38.1 | 0.55 | 108.7 | 0.81 | 0.69 | 0.93 | +184.9% | 102.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 1172 | 39.4 | 6412 | 282.3 | 201.5 | 5657 | +447.0% | 26.0 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 765.2 | 68.8 | 435.3 | 20.3 | 50.7 | 56.2 | -43.1% | 6.51 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1725 | 258.7 | 2183 | 303.7 | 282.1 | 110.1 | +26.6% | 1.62 | no difference |
| P+ vs H3 | q2_p50_us | 529.9 | 19.6 | 410.0 | 15.9 | 17.8 | 54.7 | -22.6% | 6.73 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1183 | 140.0 | 1672 | 288.4 | 226.7 | 487.8 | +41.3% | 2.16 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 2140 | 161.5 | 999.3 | 60.9 | 122.1 | 266.0 | -53.3% | 9.35 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 6286 | 613.1 | 3531 | 170.6 | 450.0 | 768.0 | -43.8% | 6.12 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1857 | 117.7 | 1009 | 55.3 | 91.9 | 273.6 | -45.7% | 9.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 6530 | 1124 | 3217 | 395.8 | 842.3 | 139.9 | -50.7% | 3.93 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 420.0 | 18.7 | 109.9 | 0.91 | 13.3 | 244.6 | -73.8% | 23.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.8 | 0.01 | 108.7 | 0.81 | 0.57 | 0.82 | +137.2% | 110.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -255.2 | 14.0 | 6412 | 282.3 | 199.9 | 5602 | -2612.8% | 33.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 311.4 | 26.4 | 784.5 | 35.9 | 31.5 | 105.0 | +151.9% | 15.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 585.9 | 36.3 | 1425 | 110.1 | 81.9 | 153.9 | +143.3% | 10.2 | **N better** |
| N vs P+ | q2_p50_us | 187.3 | 7.99 | 548.0 | 35.3 | 25.6 | 12.5 | +192.5% | 14.1 | **N better** |
| N vs P+ | q2_p99_us | 1133 | 169.1 | 1145 | 216.8 | 194.4 | 710.1 | +1.0% | 0.06 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 526.5 | 24.0 | 2170 | 111.8 | 80.9 | 47.9 | +312.2% | 20.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 1669 | 116.8 | 6213 | 546.6 | 395.2 | 1134 | +272.2% | 11.5 | **N better** |
| N vs P+ | c4_q2_p50_us | 302.8 | 18.0 | 1824 | 122.9 | 87.9 | 36.6 | +502.2% | 17.3 | **N better** |
| N vs P+ | c4_q2_p99_us | 1870 | 336.0 | 5912 | 905.6 | 683.0 | 994.4 | +216.2% | 5.92 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 304.1 | 8.24 | 424.3 | 18.7 | 14.4 | 296.3 | +39.5% | 8.33 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 37.5 | 0.31 | 45.6 | 0.03 | 0.22 | 0.52 | +21.5% | 37.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 1097 | 64.3 | -297.9 | 18.1 | 47.2 | 840.8 | -127.2% | 29.5 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 311.4 | 26.4 | 429.5 | 16.8 | 22.1 | 105.0 | +37.9% | 5.34 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 585.9 | 36.3 | 1947 | 88.7 | 67.8 | 746.7 | +232.3% | 20.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 187.3 | 7.99 | 408.0 | 13.2 | 10.9 | 10.1 | +117.8% | 20.2 | **N better** |
| N vs H3 | q2_p99_us | 1133 | 169.1 | 1436 | 204.8 | 187.8 | 1002 | +26.7% | 1.61 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p50_us | 526.5 | 24.0 | 1026 | 38.4 | 32.0 | 36.7 | +94.9% | 15.6 | **N better** |
| N vs H3 | c4_q1_p99_us | 1669 | 116.8 | 3773 | 393.1 | 290.0 | 370.3 | +126.0% | 7.26 | **N better** |
| N vs H3 | c4_q2_p50_us | 302.8 | 18.0 | 1011 | 52.6 | 39.3 | 124.1 | +234.0% | 18.0 | **N better** |
| N vs H3 | c4_q2_p99_us | 1870 | 336.0 | 3428 | 270.3 | 305.0 | 825.4 | +83.3% | 5.11 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 304.1 | 8.24 | 108.9 | 0.26 | 5.83 | 192.5 | -64.2% | 33.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 37.5 | 0.31 | 108.1 | 0.90 | 0.67 | 0.52 | +188.1% | 105.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 1097 | 64.3 | 6360 | 300.9 | 217.6 | 5220 | +479.8% | 24.2 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 784.5 | 35.9 | 429.5 | 16.8 | 28.0 | 4.21 | -45.3% | 12.7 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1425 | 110.1 | 1947 | 88.7 | 100.0 | 758.3 | +36.6% | 5.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 548.0 | 35.3 | 408.0 | 13.2 | 26.6 | 7.36 | -25.5% | 5.26 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1145 | 216.8 | 1436 | 204.8 | 210.9 | 707.1 | +25.5% | 1.38 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2170 | 111.8 | 1026 | 38.4 | 83.6 | 39.6 | -52.7% | 13.7 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6213 | 546.6 | 3773 | 393.1 | 476.1 | 1186 | -39.3% | 5.12 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1824 | 122.9 | 1011 | 52.6 | 94.6 | 129.2 | -44.5% | 8.59 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5912 | 905.6 | 3428 | 270.3 | 668.3 | 604.0 | -42.0% | 3.72 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 424.3 | 18.7 | 108.9 | 0.26 | 13.2 | 225.5 | -74.3% | 23.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.6 | 0.03 | 108.1 | 0.90 | 0.63 | 0.10 | +137.1% | 98.5 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -297.9 | 18.1 | 6360 | 300.9 | 213.1 | 5169 | -2234.9% | 31.2 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 305.2 | 10.9 | 743.0 | 30.5 | 22.9 | 84.9 | +143.4% | 19.1 | **N better** |
| N vs P+ | q1_p99_us | 646.3 | 66.9 | 1504 | 121.1 | 97.8 | 383.8 | +132.7% | 8.77 | **N better** |
| N vs P+ | q2_p50_us | 193.4 | 5.46 | 524.2 | 8.53 | 7.16 | 100.0 | +171.0% | 46.2 | **N better** |
| N vs P+ | q2_p99_us | 1164 | 57.2 | 1095 | 106.9 | 85.7 | 351.0 | -5.9% | 0.80 | BELOW FLOOR |
| N vs P+ | c4_q1_p50_us | 506.7 | 21.9 | 2128 | 60.0 | 45.2 | 236.6 | +320.0% | 35.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 1808 | 244.3 | 5804 | 468.2 | 373.4 | 2060 | +221.0% | 10.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 298.1 | 23.7 | 1758 | 49.3 | 38.7 | 70.4 | +489.7% | 37.8 | **N better** |
| N vs P+ | c4_q2_p99_us | 2152 | 375.5 | 5413 | 448.1 | 413.4 | 1066 | +151.5% | 7.89 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 305.3 | 8.10 | 427.6 | 20.2 | 15.4 | 312.8 | +40.1% | 7.96 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 38.2 | 0.24 | 45.7 | 0.03 | 0.17 | 1.21 | +19.7% | 43.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 1177 | 118.0 | -272.4 | 10.9 | 83.8 | 666.1 | -123.1% | 17.3 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 305.2 | 10.9 | 457.0 | 28.5 | 21.6 | 57.2 | +49.7% | 7.04 | **N better** |
| N vs H3 | q1_p99_us | 646.3 | 66.9 | 2231 | 138.6 | 108.8 | 154.8 | +245.2% | 14.6 | **N better** |
| N vs H3 | q2_p50_us | 193.4 | 5.46 | 428.5 | 24.1 | 17.5 | 62.3 | +121.5% | 13.5 | **N better** |
| N vs H3 | q2_p99_us | 1164 | 57.2 | 1560 | 37.9 | 48.5 | 351.7 | +34.0% | 8.16 | **N better** |
| N vs H3 | c4_q1_p50_us | 506.7 | 21.9 | 999.7 | 51.3 | 39.4 | 196.3 | +97.3% | 12.5 | **N better** |
| N vs H3 | c4_q1_p99_us | 1808 | 244.3 | 3864 | 400.2 | 331.5 | 284.3 | +113.7% | 6.20 | **N better** |
| N vs H3 | c4_q2_p50_us | 298.1 | 23.7 | 956.9 | 47.1 | 37.3 | 181.8 | +221.0% | 17.7 | **N better** |
| N vs H3 | c4_q2_p99_us | 2152 | 375.5 | 3476 | 275.4 | 329.3 | 734.8 | +61.5% | 4.02 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 305.3 | 8.10 | 109.3 | 0.64 | 5.75 | 191.2 | -64.2% | 34.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 38.2 | 0.24 | 109.4 | 0.13 | 0.19 | 1.37 | +186.8% | 366.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 1177 | 118.0 | 6574 | 378.5 | 280.3 | 5455 | +458.3% | 19.2 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 743.0 | 30.5 | 457.0 | 28.5 | 29.5 | 89.8 | -38.5% | 9.69 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1504 | 121.1 | 2231 | 138.6 | 130.1 | 373.8 | +48.4% | 5.59 | **P+ better** |
| P+ vs H3 | q2_p50_us | 524.2 | 8.53 | 428.5 | 24.1 | 18.1 | 113.3 | -18.3% | 5.30 | BELOW FLOOR |
| P+ vs H3 | q2_p99_us | 1095 | 106.9 | 1560 | 37.9 | 80.2 | 85.1 | +42.4% | 5.79 | **P+ better** |
| P+ vs H3 | c4_q1_p50_us | 2128 | 60.0 | 999.7 | 51.3 | 55.8 | 306.4 | -53.0% | 20.2 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 5804 | 468.2 | 3864 | 400.2 | 435.5 | 2074 | -33.4% | 4.45 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1758 | 49.3 | 956.9 | 47.1 | 48.2 | 170.0 | -45.6% | 16.6 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5413 | 448.1 | 3476 | 275.4 | 371.9 | 1248 | -35.8% | 5.21 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 427.6 | 20.2 | 109.3 | 0.64 | 14.3 | 247.9 | -74.4% | 22.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.7 | 0.03 | 109.4 | 0.13 | 0.09 | 0.68 | +139.5% | 686.9 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -272.4 | 10.9 | 6574 | 378.5 | 267.7 | 5424 | -2513.6% | 25.6 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 305.4 | 26.5 | 763.4 | 35.9 | 31.6 | 70.5 | +149.9% | 14.5 | **N better** |
| N vs P+ | q1_p99_us | 641.6 | 98.4 | 1763 | 432.1 | 313.3 | 263.6 | +174.8% | 3.58 | **N better** |
| N vs P+ | q2_p50_us | 190.1 | 14.1 | 553.8 | 17.0 | 15.6 | 45.1 | +191.3% | 23.3 | **N better** |
| N vs P+ | q2_p99_us | 1264 | 153.8 | 1355 | 223.0 | 191.5 | 337.2 | +7.1% | 0.47 | BELOW FLOOR |
| N vs P+ | c4_q1_p50_us | 518.7 | 33.7 | 2175 | 109.8 | 81.2 | 352.7 | +319.3% | 20.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 1854 | 143.7 | 7291 | 943.4 | 674.8 | 846.9 | +293.3% | 8.06 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 295.6 | 9.47 | 1851 | 90.3 | 64.2 | 252.2 | +526.3% | 24.2 | **N better** |
| N vs P+ | c4_q2_p99_us | 1740 | 174.5 | 6629 | 856.3 | 617.9 | 1656 | +281.0% | 7.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 304.2 | 8.20 | 428.9 | 22.4 | 16.9 | 288.8 | +41.0% | 7.39 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 37.8 | 0.33 | 45.8 | 0.04 | 0.23 | 1.04 | +20.9% | 33.7 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 1140 | 41.0 | -273.5 | 15.2 | 30.9 | 771.8 | -124.0% | 45.7 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 305.4 | 26.5 | 454.1 | 17.7 | 22.5 | 33.3 | +48.7% | 6.60 | **N better** |
| N vs H3 | q1_p99_us | 641.6 | 98.4 | 2168 | 132.7 | 116.8 | 38.8 | +237.9% | 13.1 | **N better** |
| N vs H3 | q2_p50_us | 190.1 | 14.1 | 423.1 | 15.1 | 14.6 | 22.2 | +122.5% | 16.0 | **N better** |
| N vs H3 | q2_p99_us | 1264 | 153.8 | 1897 | 370.4 | 283.6 | 396.3 | +50.0% | 2.23 | no difference |
| N vs H3 | c4_q1_p50_us | 518.7 | 33.7 | 999.5 | 40.7 | 37.4 | 133.9 | +92.7% | 12.9 | **N better** |
| N vs H3 | c4_q1_p99_us | 1854 | 143.7 | 3986 | 254.1 | 206.4 | 1562 | +115.0% | 10.3 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 295.6 | 9.47 | 997.1 | 47.3 | 34.1 | 26.5 | +237.3% | 20.6 | **N better** |
| N vs H3 | c4_q2_p99_us | 1740 | 174.5 | 3891 | 528.6 | 393.6 | 1743 | +123.6% | 5.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 304.2 | 8.20 | 109.9 | 0.63 | 5.82 | 197.3 | -63.9% | 33.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 37.8 | 0.33 | 108.5 | 0.13 | 0.25 | 1.22 | +186.7% | 283.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 1140 | 41.0 | 6427 | 375.1 | 266.8 | 5549 | +463.7% | 19.8 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 763.4 | 35.9 | 454.1 | 17.7 | 28.3 | 62.2 | -40.5% | 10.9 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1763 | 432.1 | 2168 | 132.7 | 319.6 | 264.9 | +23.0% | 1.27 | no difference |
| P+ vs H3 | q2_p50_us | 553.8 | 17.0 | 423.1 | 15.1 | 16.1 | 47.0 | -23.6% | 8.13 | **H3 better** |
| P+ vs H3 | q2_p99_us | 1355 | 223.0 | 1897 | 370.4 | 305.7 | 499.3 | +40.0% | 1.77 | no difference |
| P+ vs H3 | c4_q1_p50_us | 2175 | 109.8 | 999.5 | 40.7 | 82.8 | 371.0 | -54.0% | 14.2 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 7291 | 943.4 | 3986 | 254.1 | 690.9 | 1409 | -45.3% | 4.78 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1851 | 90.3 | 997.1 | 47.3 | 72.1 | 253.6 | -46.1% | 11.8 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6629 | 856.3 | 3891 | 528.6 | 711.6 | 602.3 | -41.3% | 3.85 | **H3 better** |
| P+ vs H3 | view_state_bytes_per_key_read | 428.9 | 22.4 | 109.9 | 0.63 | 15.9 | 211.1 | -74.4% | 20.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.8 | 0.04 | 108.5 | 0.13 | 0.09 | 0.66 | +137.1% | 661.6 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -273.5 | 15.2 | 6427 | 375.1 | 265.4 | 5509 | -2449.8% | 25.2 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1127 | 71.1 | 759.6 | 13.7 | 51.2 | 2.84 | -32.6% | 7.18 | **P+ better** |
| N vs P+ | q1_p99_us | 2689 | 232.5 | 1538 | 105.4 | 180.5 | 231.1 | -42.8% | 6.38 | **P+ better** |
| N vs P+ | q2_p50_us | 257.7 | 7.39 | 572.1 | 17.0 | 13.1 | 15.0 | +122.0% | 24.0 | **N better** |
| N vs P+ | q2_p99_us | 3573 | 854.5 | 1262 | 174.5 | 616.7 | 830.5 | -64.7% | 3.75 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 2969 | 235.1 | 2268 | 85.2 | 176.8 | 62.8 | -23.6% | 3.96 | **P+ better** |
| N vs P+ | c4_q1_p99_us | 9963 | 1527 | 7064 | 1106 | 1333 | 1227 | -29.1% | 2.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 254.2 | 7.79 | 1940 | 42.9 | 30.8 | 27.9 | +662.9% | 54.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 4999 | 850.6 | 6431 | 1011 | 934.2 | 3072 | +28.7% | 1.53 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 729.4 | 138.1 | 811.4 | 155.5 | 147.0 | 2177 | +11.2% | 0.56 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 272.0 | 1.62 | 478.2 | 0.03 | 1.14 | 0.51 | +75.8% | 180.3 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 943.9 | 158.3 | 8581 | 1786 | 1268 | 19378 | +809.1% | 6.03 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 1127 | 71.1 | 1232 | 82.0 | 76.7 | 17.4 | +9.3% | 1.37 | no difference |
| N vs H3 | q1_p99_us | 2689 | 232.5 | 4896 | 813.6 | 598.3 | 2562 | +82.0% | 3.69 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 257.7 | 7.39 | 1188 | 78.3 | 55.6 | 14.9 | +360.9% | 16.7 | **N better** |
| N vs H3 | q2_p99_us | 3573 | 854.5 | 4189 | 568.5 | 725.7 | 1118 | +17.2% | 0.85 | BELOW FLOOR |
| N vs H3 | c4_q1_p50_us | 2969 | 235.1 | 3748 | 227.0 | 231.1 | 111.5 | +26.3% | 3.37 | **N better** |
| N vs H3 | c4_q1_p99_us | 9963 | 1527 | 12103 | 764.6 | 1207 | 1460 | +21.5% | 1.77 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 254.2 | 7.79 | 3644 | 313.0 | 221.4 | 224.5 | +1333.2% | 15.3 | **N better** |
| N vs H3 | c4_q2_p99_us | 4999 | 850.6 | 11442 | 839.4 | 845.0 | 2858 | +128.9% | 7.63 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 729.4 | 138.1 | 124.9 | 4.81 | 97.7 | 1478 | -82.9% | 6.19 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 272.0 | 1.62 | 502.4 | 1.41 | 1.52 | 2.37 | +84.7% | 151.7 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 943.9 | 158.3 | 9321 | 1953 | 1385 | 20448 | +887.5% | 6.05 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 759.6 | 13.7 | 1232 | 82.0 | 58.8 | 17.2 | +62.2% | 8.04 | **P+ better** |
| P+ vs H3 | q1_p99_us | 1538 | 105.4 | 4896 | 813.6 | 580.1 | 2573 | +218.4% | 5.79 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 572.1 | 17.0 | 1188 | 78.3 | 56.6 | 14.6 | +107.6% | 10.9 | **P+ better** |
| P+ vs H3 | q2_p99_us | 1262 | 174.5 | 4189 | 568.5 | 420.5 | 748.6 | +231.9% | 6.96 | **P+ better** |
| P+ vs H3 | c4_q1_p50_us | 2268 | 85.2 | 3748 | 227.0 | 171.4 | 124.2 | +65.3% | 8.63 | **P+ better** |
| P+ vs H3 | c4_q1_p99_us | 7064 | 1106 | 12103 | 764.6 | 950.8 | 1316 | +71.3% | 5.30 | **P+ better** |
| P+ vs H3 | c4_q2_p50_us | 1940 | 42.9 | 3644 | 313.0 | 223.4 | 226.0 | +87.9% | 7.63 | **P+ better** |
| P+ vs H3 | c4_q2_p99_us | 6431 | 1011 | 11442 | 839.4 | 929.1 | 1131 | +77.9% | 5.39 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 811.4 | 155.5 | 124.9 | 4.81 | 110.0 | 1600 | -84.6% | 6.24 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 478.2 | 0.03 | 502.4 | 1.41 | 1.00 | 2.36 | +5.1% | 24.2 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 8581 | 1786 | 9321 | 1953 | 1871 | 28076 | +8.6% | 0.40 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 999.4 | 92.0 | 754.9 | 36.2 | 69.9 | 40.3 | -24.5% | 3.50 | **P+ better** |
| N vs P+ | q1_p99_us | 2125 | 285.7 | 1540 | 271.5 | 278.7 | 133.9 | -27.5% | 2.10 | no difference |
| N vs P+ | q2_p50_us | 241.3 | 11.2 | 563.4 | 21.6 | 17.2 | 51.6 | +133.5% | 18.7 | **N better** |
| N vs P+ | q2_p99_us | 3407 | 289.6 | 1192 | 195.6 | 247.1 | 150.8 | -65.0% | 8.96 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 2672 | 203.9 | 2189 | 152.3 | 180.0 | 150.0 | -18.1% | 2.68 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 7297 | 1037 | 6796 | 395.8 | 785.0 | 385.7 | -6.9% | 0.64 | no difference |
| N vs P+ | c4_q2_p50_us | 224.5 | 13.8 | 1901 | 120.0 | 85.4 | 43.8 | +746.7% | 19.6 | **N better** |
| N vs P+ | c4_q2_p99_us | 3853 | 561.2 | 6974 | 1031 | 830.3 | 2199 | +81.0% | 3.76 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 733.7 | 139.1 | 811.5 | 148.4 | 143.8 | 2149 | +10.6% | 0.54 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 270.0 | 1.92 | 476.2 | 0.03 | 1.36 | 0.06 | +76.4% | 151.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 786.7 | 246.6 | 8455 | 1759 | 1256 | 18861 | +974.7% | 6.11 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 999.4 | 92.0 | 1223 | 84.4 | 88.3 | 48.3 | +22.4% | 2.53 | no difference |
| N vs H3 | q1_p99_us | 2125 | 285.7 | 4937 | 412.3 | 354.7 | 844.1 | +132.3% | 7.93 | **N better** |
| N vs H3 | q2_p50_us | 241.3 | 11.2 | 1158 | 90.1 | 64.2 | 83.5 | +379.9% | 14.3 | **N better** |
| N vs H3 | q2_p99_us | 3407 | 289.6 | 4323 | 773.6 | 584.1 | 1284 | +26.9% | 1.57 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 2672 | 203.9 | 3322 | 478.8 | 368.0 | 303.7 | +24.3% | 1.77 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 7297 | 1037 | 10534 | 997.8 | 1018 | 888.2 | +44.4% | 3.18 | **N better** |
| N vs H3 | c4_q2_p50_us | 224.5 | 13.8 | 3290 | 456.2 | 322.7 | 437.4 | +1365.7% | 9.50 | **N better** |
| N vs H3 | c4_q2_p99_us | 3853 | 561.2 | 9900 | 1138 | 897.3 | 2559 | +156.9% | 6.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 733.7 | 139.1 | 125.7 | 4.53 | 98.4 | 1475 | -82.9% | 6.18 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 270.0 | 1.92 | 500.8 | 1.52 | 1.73 | 0.80 | +85.5% | 133.4 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 786.7 | 246.6 | 9377 | 1896 | 1352 | 20495 | +1092.0% | 6.35 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 754.9 | 36.2 | 1223 | 84.4 | 64.9 | 62.9 | +62.0% | 7.21 | **P+ better** |
| P+ vs H3 | q1_p99_us | 1540 | 271.5 | 4937 | 412.3 | 349.1 | 844.2 | +220.6% | 9.73 | **P+ better** |
| P+ vs H3 | q2_p50_us | 563.4 | 21.6 | 1158 | 90.1 | 65.5 | 95.1 | +105.5% | 9.07 | **P+ better** |
| P+ vs H3 | q2_p99_us | 1192 | 195.6 | 4323 | 773.6 | 564.2 | 1284 | +262.7% | 5.55 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2189 | 152.3 | 3322 | 478.8 | 355.3 | 266.7 | +51.8% | 3.19 | **P+ better** |
| P+ vs H3 | c4_q1_p99_us | 6796 | 395.8 | 10534 | 997.8 | 759.0 | 944.8 | +55.0% | 4.92 | **P+ better** |
| P+ vs H3 | c4_q2_p50_us | 1901 | 120.0 | 3290 | 456.2 | 333.6 | 438.6 | +73.1% | 4.17 | **P+ better** |
| P+ vs H3 | c4_q2_p99_us | 6974 | 1031 | 9900 | 1138 | 1086 | 2118 | +41.9% | 2.69 | no difference |
| P+ vs H3 | view_state_bytes_per_key_read | 811.5 | 148.4 | 125.7 | 4.53 | 105.0 | 1564 | -84.5% | 6.53 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 476.2 | 0.03 | 500.8 | 1.52 | 1.07 | 0.80 | +5.2% | 23.0 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 8455 | 1759 | 9377 | 1896 | 1829 | 27834 | +10.9% | 0.50 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1040 | 59.6 | 827.3 | 84.6 | 73.2 | 15.8 | -20.5% | 2.91 | no difference |
| N vs P+ | q1_p99_us | 2638 | 491.6 | 1944 | 315.7 | 413.1 | 269.1 | -26.3% | 1.68 | no difference |
| N vs P+ | q2_p50_us | 215.4 | 10.6 | 653.8 | 79.5 | 56.7 | 14.9 | +203.5% | 7.73 | **N better** |
| N vs P+ | q2_p99_us | 2793 | 521.7 | 1440 | 327.5 | 435.6 | 1407 | -48.4% | 3.10 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 2641 | 372.2 | 2311 | 157.7 | 285.8 | 112.4 | -12.5% | 1.15 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 8483 | 1287 | 7563 | 930.0 | 1123 | 2046 | -10.8% | 0.82 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 244.6 | 7.39 | 2015 | 120.9 | 85.6 | 243.8 | +724.1% | 20.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 5066 | 427.3 | 7107 | 998.4 | 767.9 | 1817 | +40.3% | 2.66 | no difference |
| N vs P+ | view_state_bytes_per_key_read | 734.7 | 141.8 | 809.6 | 158.5 | 150.4 | 2177 | +10.2% | 0.50 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 272.9 | 1.71 | 476.5 | 0.02 | 1.21 | 5.99 | +74.6% | 168.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 1028 | 219.6 | 8656 | 1828 | 1302 | 19522 | +741.7% | 5.86 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 1040 | 59.6 | 1247 | 136.7 | 105.4 | 70.0 | +19.9% | 1.96 | no difference |
| N vs H3 | q1_p99_us | 2638 | 491.6 | 4969 | 470.9 | 481.4 | 795.8 | +88.3% | 4.84 | **N better** |
| N vs H3 | q2_p50_us | 215.4 | 10.6 | 1203 | 107.4 | 76.3 | 27.2 | +458.7% | 12.9 | **N better** |
| N vs H3 | q2_p99_us | 2793 | 521.7 | 4725 | 499.5 | 510.8 | 1474 | +69.2% | 3.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 2641 | 372.2 | 3673 | 326.9 | 350.3 | 753.0 | +39.1% | 2.95 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 8483 | 1287 | 12732 | 1871 | 1606 | 1069 | +50.1% | 2.65 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 244.6 | 7.39 | 3604 | 321.7 | 227.5 | 926.5 | +1373.7% | 14.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 5066 | 427.3 | 12820 | 733.9 | 600.5 | 1053 | +153.1% | 12.9 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 734.7 | 141.8 | 126.2 | 5.71 | 100.3 | 1491 | -82.8% | 6.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 272.9 | 1.71 | 500.9 | 1.27 | 1.51 | 6.22 | +83.5% | 151.2 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 1028 | 219.6 | 9427 | 1948 | 1386 | 20658 | +816.7% | 6.06 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 827.3 | 84.6 | 1247 | 136.7 | 113.7 | 71.7 | +50.7% | 3.69 | **P+ better** |
| P+ vs H3 | q1_p99_us | 1944 | 315.7 | 4969 | 470.9 | 400.9 | 829.6 | +155.6% | 7.55 | **P+ better** |
| P+ vs H3 | q2_p50_us | 653.8 | 79.5 | 1203 | 107.4 | 94.5 | 27.9 | +84.1% | 5.82 | **P+ better** |
| P+ vs H3 | q2_p99_us | 1440 | 327.5 | 4725 | 499.5 | 422.4 | 606.5 | +228.0% | 7.78 | **P+ better** |
| P+ vs H3 | c4_q1_p50_us | 2311 | 157.7 | 3673 | 326.9 | 256.6 | 744.8 | +58.9% | 5.31 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 7563 | 930.0 | 12732 | 1871 | 1477 | 1778 | +68.4% | 3.50 | **P+ better** |
| P+ vs H3 | c4_q2_p50_us | 2015 | 120.9 | 3604 | 321.7 | 243.0 | 958.0 | +78.8% | 6.54 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 7107 | 998.4 | 12820 | 733.9 | 876.2 | 1734 | +80.4% | 6.52 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 809.6 | 158.5 | 126.2 | 5.71 | 112.1 | 1587 | -84.4% | 6.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 476.5 | 0.02 | 500.9 | 1.27 | 0.90 | 1.66 | +5.1% | 27.2 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 8656 | 1828 | 9427 | 1948 | 1889 | 28388 | +8.9% | 0.41 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1039 | 33.9 | 836.2 | 77.7 | 59.9 | 86.9 | -19.5% | 3.39 | **P+ better** |
| N vs P+ | q1_p99_us | 2255 | 289.3 | 1632 | 187.3 | 243.7 | 240.1 | -27.6% | 2.56 | no difference |
| N vs P+ | q2_p50_us | 236.6 | 13.9 | 640.8 | 67.5 | 48.7 | 63.9 | +170.8% | 8.30 | **N better** |
| N vs P+ | q2_p99_us | 2506 | 414.1 | 1509 | 35.2 | 293.9 | 1337 | -39.8% | 3.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 2668 | 114.1 | 2347 | 157.8 | 137.7 | 85.8 | -12.0% | 2.33 | no difference |
| N vs P+ | c4_q1_p99_us | 7183 | 756.0 | 7418 | 862.1 | 810.8 | 2477 | +3.3% | 0.29 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 221.1 | 14.1 | 2025 | 156.1 | 110.8 | 24.8 | +815.9% | 16.3 | **N better** |
| N vs P+ | c4_q2_p99_us | 4408 | 757.1 | 6665 | 440.3 | 619.3 | 2060 | +51.2% | 3.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | view_state_bytes_per_key_read | 730.2 | 138.2 | 800.1 | 148.4 | 143.4 | 2168 | +9.6% | 0.49 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 271.1 | 0.58 | 478.0 | 0.03 | 0.41 | 0.05 | +76.3% | 500.5 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 819.7 | 137.5 | 8485 | 1759 | 1248 | 19262 | +935.1% | 6.14 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 1039 | 33.9 | 1201 | 101.7 | 75.8 | 138.1 | +15.6% | 2.14 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q1_p99_us | 2255 | 289.3 | 4059 | 605.7 | 474.6 | 3189 | +80.0% | 3.80 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 236.6 | 13.9 | 1162 | 80.0 | 57.4 | 179.7 | +391.1% | 16.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p99_us | 2506 | 414.1 | 3553 | 331.2 | 375.0 | 1552 | +41.8% | 2.79 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 2668 | 114.1 | 3338 | 262.3 | 202.2 | 700.0 | +25.1% | 3.31 | BELOW FLOOR |
| N vs H3 | c4_q1_p99_us | 7183 | 756.0 | 10260 | 1132 | 962.7 | 2073 | +42.8% | 3.20 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 221.1 | 14.1 | 3303 | 263.8 | 186.8 | 650.1 | +1393.6% | 16.5 | **N better** |
| N vs H3 | c4_q2_p99_us | 4408 | 757.1 | 9931 | 1107 | 948.3 | 843.0 | +125.3% | 5.82 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 730.2 | 138.2 | 125.0 | 5.37 | 97.8 | 1485 | -82.9% | 6.19 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 271.1 | 0.58 | 503.9 | 1.65 | 1.24 | 2.77 | +85.9% | 188.0 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 819.7 | 137.5 | 9502 | 1987 | 1408 | 20838 | +1059.2% | 6.16 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 836.2 | 77.7 | 1201 | 101.7 | 90.5 | 162.9 | +43.7% | 4.04 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p99_us | 1632 | 187.3 | 4059 | 605.7 | 448.3 | 3196 | +148.8% | 5.42 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 640.8 | 67.5 | 1162 | 80.0 | 74.0 | 190.5 | +81.3% | 7.04 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p99_us | 1509 | 35.2 | 3553 | 331.2 | 235.5 | 810.5 | +135.4% | 8.68 | **P+ better** |
| P+ vs H3 | c4_q1_p50_us | 2347 | 157.8 | 3338 | 262.3 | 216.4 | 705.2 | +42.2% | 4.58 | **P+ better** |
| P+ vs H3 | c4_q1_p99_us | 7418 | 862.1 | 10260 | 1132 | 1006 | 1730 | +38.3% | 2.82 | no difference |
| P+ vs H3 | c4_q2_p50_us | 2025 | 156.1 | 3303 | 263.8 | 216.7 | 649.9 | +63.1% | 5.90 | **P+ better** |
| P+ vs H3 | c4_q2_p99_us | 6665 | 440.3 | 9931 | 1107 | 842.4 | 2145 | +49.0% | 3.88 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | view_state_bytes_per_key_read | 800.1 | 148.4 | 125.0 | 5.37 | 105.0 | 1581 | -84.4% | 6.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 478.0 | 0.03 | 503.9 | 1.65 | 1.17 | 2.77 | +5.4% | 22.2 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 8485 | 1759 | 9502 | 1987 | 1876 | 28299 | +12.0% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1039 | 93.7 | 774.7 | 37.2 | 71.3 | 149.7 | -25.4% | 3.70 | **P+ better** |
| N vs P+ | q1_p99_us | 2435 | 280.3 | 1607 | 164.5 | 229.8 | 160.5 | -34.0% | 3.60 | **P+ better** |
| N vs P+ | q2_p50_us | 249.7 | 7.38 | 599.3 | 29.7 | 21.7 | 124.6 | +140.0% | 16.1 | **N better** |
| N vs P+ | q2_p99_us | 3646 | 807.7 | 1262 | 96.2 | 575.2 | 559.9 | -65.4% | 4.14 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 2838 | 107.3 | 2218 | 46.8 | 82.7 | 173.6 | -21.9% | 7.50 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 8759 | 973.5 | 7015 | 686.9 | 842.4 | 2132 | -19.9% | 2.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 239.7 | 14.2 | 1933 | 55.7 | 40.6 | 114.3 | +706.3% | 41.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 4394 | 302.4 | 6713 | 529.1 | 430.9 | 3146 | +52.8% | 5.38 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 732.0 | 139.1 | 805.1 | 151.6 | 145.5 | 2169 | +10.0% | 0.50 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 272.1 | 1.18 | 477.3 | 0.08 | 0.83 | 0.44 | +75.4% | 245.9 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 867.5 | 124.1 | 8857 | 1989 | 1409 | 19745 | +921.0% | 5.67 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 1039 | 93.7 | 1277 | 89.9 | 91.8 | 43.1 | +22.9% | 2.59 | no difference |
| N vs H3 | q1_p99_us | 2435 | 280.3 | 5134 | 540.2 | 430.3 | 2300 | +110.8% | 6.27 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 249.7 | 7.38 | 1251 | 83.2 | 59.1 | 53.6 | +400.9% | 17.0 | **N better** |
| N vs H3 | q2_p99_us | 3646 | 807.7 | 4553 | 412.5 | 641.3 | 1678 | +24.9% | 1.41 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 2838 | 107.3 | 3420 | 150.8 | 130.9 | 428.5 | +20.5% | 4.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 8759 | 973.5 | 10782 | 1087 | 1032 | 2082 | +23.1% | 1.96 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 239.7 | 14.2 | 3504 | 221.7 | 157.1 | 641.7 | +1361.8% | 20.8 | **N better** |
| N vs H3 | c4_q2_p99_us | 4394 | 302.4 | 10744 | 1013 | 747.3 | 4039 | +144.5% | 8.50 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 732.0 | 139.1 | 125.9 | 4.77 | 98.4 | 1484 | -82.8% | 6.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 272.1 | 1.18 | 501.2 | 1.12 | 1.15 | 1.87 | +84.2% | 199.5 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 867.5 | 124.1 | 9407 | 1914 | 1356 | 20840 | +984.5% | 6.30 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 774.7 | 37.2 | 1277 | 89.9 | 68.8 | 155.7 | +64.8% | 7.29 | **P+ better** |
| P+ vs H3 | q1_p99_us | 1607 | 164.5 | 5134 | 540.2 | 399.3 | 2300 | +219.4% | 8.83 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 599.3 | 29.7 | 1251 | 83.2 | 62.5 | 135.6 | +108.8% | 10.4 | **P+ better** |
| P+ vs H3 | q2_p99_us | 1262 | 96.2 | 4553 | 412.5 | 299.5 | 1745 | +260.7% | 11.0 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q1_p50_us | 2218 | 46.8 | 3420 | 150.8 | 111.7 | 395.8 | +54.2% | 10.8 | **P+ better** |
| P+ vs H3 | c4_q1_p99_us | 7015 | 686.9 | 10782 | 1087 | 909.4 | 936.5 | +53.7% | 4.14 | **P+ better** |
| P+ vs H3 | c4_q2_p50_us | 1933 | 55.7 | 3504 | 221.7 | 161.7 | 646.0 | +81.3% | 9.72 | **P+ better** |
| P+ vs H3 | c4_q2_p99_us | 6713 | 529.1 | 10744 | 1013 | 807.9 | 2558 | +60.1% | 4.99 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 805.1 | 151.6 | 125.9 | 4.77 | 107.3 | 1584 | -84.4% | 6.33 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 477.3 | 0.08 | 501.2 | 1.12 | 0.79 | 1.85 | +5.0% | 30.2 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 8857 | 1989 | 9407 | 1914 | 1952 | 28565 | +6.2% | 0.28 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

