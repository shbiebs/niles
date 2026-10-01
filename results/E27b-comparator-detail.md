# E27b — every cell

*Generated with `results/E27b-comparator.md`, from the same point files. One row per (pair, metric) per point: medians and MADs over the ten measured runs, pooled MAD, floor (3 × pooled MAD of the warm-up runs), relative change (B − A)/A, MADs apart, verdict and, when refused, why.*

## `multi`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 165.6 | 12.9 | 585.5 | 66.5 | 47.9 | 64.0 | +253.5% | 8.77 | **N better** |
| N vs P+ | q1_p99_us | 438.0 | 75.4 | 2531 | 1040 | 737.0 | 241.6 | +477.8% | 2.84 | no difference |
| N vs P+ | q2_p50_us | 10216 | 190.3 | 428.5 | 55.7 | 140.2 | 1127 | -95.8% | 69.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 156.9 | 21.8 | 255.4 | 19.5 | 20.7 | 104.9 | +62.7% | 4.76 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q4_p50_us | 2158 | 43.5 | 1307 | 47.2 | 45.4 | 99.0 | -39.4% | 18.7 | **P+ better** |
| N vs P+ | q5_p50_us | 2246 | 86.7 | 2834 | 150.0 | 122.5 | 223.1 | +26.2% | 4.80 | **N better** |
| N vs P+ | q6_p50_us | 3720 | 97.8 | 3324 | 119.6 | 109.3 | 143.6 | -10.6% | 3.63 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 129.2 | 26.7 | 630.0 | 65.8 | 50.2 | 117.6 | +387.5% | 9.97 | **N better** |
| N vs P+ | c2_q1_p99_us | 1119 | 756.5 | 5822 | 931.5 | 848.5 | 773.6 | +420.4% | 5.54 | **N better** |
| N vs P+ | c2_q2_p50_us | 10372 | 324.6 | 519.7 | 34.2 | 230.8 | 2862 | -95.0% | 42.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 136.1 | 13.3 | 1723 | 139.8 | 99.3 | 39.5 | +1166.2% | 16.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 7525 | 829.7 | 9393 | 1415 | 1160 | 5892 | +24.8% | 1.61 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 17965 | 1347 | 1480 | 125.3 | 956.7 | 918.6 | -91.8% | 17.2 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 36943 | 6160 | 8829 | 2426 | 4681 | 10366 | -76.1% | 6.01 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 587.7 | 42.3 | 809.6 | 58.2 | 50.9 | 122.9 | +37.8% | 4.36 | **N better** |
| N vs P+ | write_p99_us | 1652 | 895.7 | 2674 | 227.8 | 653.5 | 740.2 | +61.9% | 1.56 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1483 | 230.0 | 1024 | 68.7 | 169.7 | 292.6 | -30.9% | 2.70 | no difference |
| N vs P+ | view_state_bytes_per_key_read | 291.3 | 12.3 | 817.1 | 9.60 | 11.0 | 326.5 | +180.5% | 47.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 37.2 | 0.34 | 44.6 | 0.04 | 0.24 | 0.21 | +19.9% | 30.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 29469 | 1513 | -13537 | 934.9 | 1258 | 27326 | -145.9% | 34.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 165.6 | 12.9 | 100.9 | 8.61 | 11.0 | 25.3 | -39.1% | 5.88 | **P better** |
| N vs P | q1_p99_us | 438.0 | 75.4 | 468.5 | 124.4 | 102.9 | 100.2 | +7.0% | 0.30 | BELOW FLOOR |
| N vs P | q2_p50_us | 10216 | 190.3 | 183.7 | 20.1 | 135.3 | 1113 | -98.2% | 74.1 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 156.9 | 21.8 | 223.4 | 33.2 | 28.1 | 153.1 | +42.4% | 2.37 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q4_p50_us | 2158 | 43.5 | 233.5 | 48.0 | 45.8 | 12.2 | -89.2% | 42.0 | **P better** |
| N vs P | q5_p50_us | 2246 | 86.7 | 125.1 | 19.9 | 62.9 | 164.5 | -94.4% | 33.7 | **P better** |
| N vs P | q6_p50_us | 3720 | 97.8 | 543.0 | 69.3 | 84.8 | 117.7 | -85.4% | 37.5 | **P better** |
| N vs P | c2_q1_p50_us | 129.2 | 26.7 | 85.3 | 32.7 | 29.9 | 30.5 | -34.0% | 1.47 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 1119 | 756.5 | 1866 | 123.0 | 541.9 | 342.1 | +66.8% | 1.38 | no difference |
| N vs P | c2_q2_p50_us | 10372 | 324.6 | 150.8 | 18.3 | 229.9 | 2863 | -98.5% | 44.5 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p50_us | 136.1 | 13.3 | 152.0 | 58.6 | 42.5 | 25.6 | +11.7% | 0.37 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 7525 | 829.7 | 2227 | 216.5 | 606.3 | 1675 | -70.4% | 8.74 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 17965 | 1347 | 267.8 | 20.4 | 952.7 | 905.2 | -98.5% | 18.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 36943 | 6160 | 2503 | 321.8 | 4362 | 9455 | -93.2% | 7.90 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 587.7 | 42.3 | 15108 | 411.2 | 292.3 | 395.7 | +2470.5% | 49.7 | **N better** |
| N vs P | write_p99_us | 1652 | 895.7 | 21455 | 3362 | 2461 | 2098 | +1198.9% | 8.05 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1483 | 230.0 | 63.1 | 4.52 | 162.6 | 252.1 | -95.7% | 8.73 | **N better** |
| N vs P | view_state_bytes_per_key_read | 291.3 | 12.3 | 261.5 | 10.8 | 11.6 | 251.3 | -10.2% | 2.58 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 37.2 | 0.34 | 44.9 | 0.05 | 0.25 | 0.21 | +20.8% | 31.6 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 29469 | 1513 | -6179 | 421.3 | 1111 | 24884 | -121.0% | 32.1 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 585.5 | 66.5 | 100.9 | 8.61 | 47.4 | 59.9 | -82.8% | 10.2 | **P better** |
| P+ vs P | q1_p99_us | 2531 | 1040 | 468.5 | 124.4 | 740.4 | 222.0 | -81.5% | 2.79 | no difference |
| P+ vs P | q2_p50_us | 428.5 | 55.7 | 183.7 | 20.1 | 41.9 | 182.4 | -57.1% | 5.85 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 255.4 | 19.5 | 223.4 | 33.2 | 27.2 | 184.2 | -12.5% | 1.17 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q4_p50_us | 1307 | 47.2 | 233.5 | 48.0 | 47.6 | 99.7 | -82.1% | 22.6 | **P better** |
| P+ vs P | q5_p50_us | 2834 | 150.0 | 125.1 | 19.9 | 107.0 | 150.9 | -95.6% | 25.3 | **P better** |
| P+ vs P | q6_p50_us | 3324 | 119.6 | 543.0 | 69.3 | 97.8 | 89.1 | -83.7% | 28.5 | **P better** |
| P+ vs P | c2_q1_p50_us | 630.0 | 65.8 | 85.3 | 32.7 | 52.0 | 121.1 | -86.5% | 10.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5822 | 931.5 | 1866 | 123.0 | 664.4 | 714.8 | -67.9% | 5.95 | **P better** |
| P+ vs P | c2_q2_p50_us | 519.7 | 34.2 | 150.8 | 18.3 | 27.5 | 91.9 | -71.0% | 13.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1723 | 139.8 | 152.0 | 58.6 | 107.2 | 30.2 | -91.2% | 14.7 | **P better** |
| P+ vs P | c4_q1_p99_us | 9393 | 1415 | 2227 | 216.5 | 1012 | 6121 | -76.3% | 7.08 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1480 | 125.3 | 267.8 | 20.4 | 89.8 | 209.4 | -81.9% | 13.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 8829 | 2426 | 2503 | 321.8 | 1730 | 4469 | -71.6% | 3.66 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 809.6 | 58.2 | 15108 | 411.2 | 293.7 | 411.6 | +1766.0% | 48.7 | **P+ better** |
| P+ vs P | write_p99_us | 2674 | 227.8 | 21455 | 3362 | 2383 | 1994 | +702.4% | 7.88 | **P+ better** |
| P+ vs P | commits_per_s | 1024 | 68.7 | 63.1 | 4.52 | 48.7 | 148.5 | -93.8% | 19.7 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 817.1 | 9.60 | 261.5 | 10.8 | 10.2 | 302.7 | -68.0% | 54.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | pss_mib | 44.6 | 0.04 | 44.9 | 0.05 | 0.04 | 0.09 | +0.8% | 7.90 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -13537 | 934.9 | -6179 | 421.3 | 725.1 | 14058 | -54.4% | 10.1 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 100.9 | 8.61 | 93.4 | 7.77 | 8.20 | 8.32 | -7.4% | 0.91 | BELOW FLOOR |
| P vs M | q1_p99_us | 468.5 | 124.4 | 488.9 | 129.7 | 127.1 | 135.8 | +4.4% | 0.16 | BELOW FLOOR |
| P vs M | q2_p50_us | 183.7 | 20.1 | 165.4 | 13.3 | 17.0 | 22.6 | -10.0% | 1.08 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 223.4 | 33.2 | 199.7 | 36.0 | 34.6 | 276.5 | -10.6% | 0.69 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 233.5 | 48.0 | 217.2 | 24.5 | 38.1 | 23.8 | -7.0% | 0.43 | BELOW FLOOR |
| P vs M | q5_p50_us | 125.1 | 19.9 | 127.5 | 11.9 | 16.4 | 10.1 | +1.9% | 0.14 | BELOW FLOOR |
| P vs M | q6_p50_us | 543.0 | 69.3 | 552.3 | 23.8 | 51.8 | 27.0 | +1.7% | 0.18 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 85.3 | 32.7 | 93.2 | 7.72 | 23.8 | 29.8 | +9.3% | 0.34 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1866 | 123.0 | 1239 | 335.5 | 252.7 | 126.4 | -33.6% | 2.48 | no difference |
| P vs M | c2_q2_p50_us | 150.8 | 18.3 | 173.9 | 17.4 | 17.9 | 90.9 | +15.2% | 1.29 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 152.0 | 58.6 | 166.6 | 26.5 | 45.5 | 190.2 | +9.6% | 0.32 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2227 | 216.5 | 2202 | 399.5 | 321.3 | 1671 | -1.1% | 0.08 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 267.8 | 20.4 | 265.6 | 43.1 | 33.7 | 279.8 | -0.8% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p99_us | 2503 | 321.8 | 2198 | 349.3 | 335.9 | 1108 | -12.2% | 0.91 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 15108 | 411.2 | 15180 | 275.6 | 350.0 | 1937 | +0.5% | 0.21 | BELOW FLOOR |
| P vs M | write_p99_us | 21455 | 3362 | 22420 | 2325 | 2891 | 1983 | +4.5% | 0.33 | BELOW FLOOR |
| P vs M | commits_per_s | 63.1 | 4.52 | 60.7 | 2.81 | 3.76 | 2.74 | -3.7% | 0.61 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 261.5 | 10.8 | 261.5 | 14.1 | 12.6 | 219.4 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.9 | 0.05 | 44.8 | 0.03 | 0.04 | 0.08 | -0.3% | 4.15 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -6179 | 421.3 | -3852 | 241.2 | 343.3 | 7032 | -37.7% | 6.78 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 165.6 | 12.9 | 222.3 | 17.8 | 15.6 | 30.8 | +34.2% | 3.65 | **N better** |
| N vs H3 | q1_p99_us | 438.0 | 75.4 | 732.6 | 115.1 | 97.3 | 222.1 | +67.3% | 3.03 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 10216 | 190.3 | 231.5 | 36.7 | 137.1 | 1113 | -97.7% | 72.9 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 156.9 | 21.8 | 247.7 | 25.1 | 23.5 | 72.5 | +57.8% | 3.86 | **N better** |
| N vs H3 | q4_p50_us | 2158 | 43.5 | 1355 | 28.4 | 36.8 | 44.4 | -37.2% | 21.8 | **H3 better** |
| N vs H3 | q5_p50_us | 2246 | 86.7 | 2908 | 68.2 | 78.0 | 200.8 | +29.5% | 8.49 | **N better** |
| N vs H3 | q6_p50_us | 3720 | 97.8 | 3705 | 163.2 | 134.6 | 123.2 | -0.4% | 0.11 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 129.2 | 26.7 | 181.6 | 10.3 | 20.2 | 18.9 | +40.5% | 2.59 | no difference |
| N vs H3 | c2_q1_p99_us | 1119 | 756.5 | 717.9 | 92.1 | 538.8 | 498.6 | -35.8% | 0.74 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 10372 | 324.6 | 188.6 | 19.5 | 229.9 | 2862 | -98.2% | 44.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 136.1 | 13.3 | 307.1 | 18.6 | 16.2 | 48.7 | +125.6% | 10.6 | **N better** |
| N vs H3 | c4_q1_p99_us | 7525 | 829.7 | 1110 | 175.7 | 599.7 | 352.0 | -85.3% | 10.7 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 17965 | 1347 | 313.1 | 40.2 | 953.0 | 916.3 | -98.3% | 18.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 36943 | 6160 | 1122 | 159.2 | 4357 | 9454 | -97.0% | 8.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 587.7 | 42.3 | 1181 | 29.5 | 36.4 | 38.0 | +100.9% | 16.3 | **N better** |
| N vs H3 | write_p99_us | 1652 | 895.7 | 4295 | 640.8 | 778.8 | 1499 | +160.0% | 3.39 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1483 | 230.0 | 704.3 | 81.5 | 172.5 | 276.8 | -52.5% | 4.51 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 291.3 | 12.3 | 107.9 | 2.22 | 8.82 | 198.2 | -63.0% | 20.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 37.2 | 0.34 | 71.1 | 0.33 | 0.34 | 0.41 | +91.1% | 100.4 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 29469 | 1513 | 13804 | 662.1 | 1168 | 27676 | -53.2% | 13.4 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 585.5 | 66.5 | 222.3 | 17.8 | 48.7 | 62.4 | -62.0% | 7.46 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2531 | 1040 | 732.6 | 115.1 | 739.6 | 297.6 | -71.0% | 2.43 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 428.5 | 55.7 | 231.5 | 36.7 | 47.2 | 181.8 | -46.0% | 4.18 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 255.4 | 19.5 | 247.7 | 25.1 | 22.5 | 125.5 | -3.0% | 0.34 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | 1307 | 47.2 | 1355 | 28.4 | 39.0 | 108.5 | +3.7% | 1.23 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 2834 | 150.0 | 2908 | 68.2 | 116.5 | 189.8 | +2.6% | 0.64 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 3324 | 119.6 | 3705 | 163.2 | 143.1 | 96.2 | +11.5% | 2.66 | no difference |
| P+ vs H3 | c2_q1_p50_us | 630.0 | 65.8 | 181.6 | 10.3 | 47.1 | 118.7 | -71.2% | 9.52 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5822 | 931.5 | 717.9 | 92.1 | 661.9 | 801.6 | -87.7% | 7.71 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 519.7 | 34.2 | 188.6 | 19.5 | 27.9 | 49.3 | -63.7% | 11.9 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1723 | 139.8 | 307.1 | 18.6 | 99.7 | 51.3 | -82.2% | 14.2 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 9393 | 1415 | 1110 | 175.7 | 1008 | 5898 | -88.2% | 8.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1480 | 125.3 | 313.1 | 40.2 | 93.0 | 253.3 | -78.8% | 12.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 8829 | 2426 | 1122 | 159.2 | 1719 | 4467 | -87.3% | 4.48 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 809.6 | 58.2 | 1181 | 29.5 | 46.1 | 119.5 | +45.9% | 8.04 | **P+ better** |
| P+ vs H3 | write_p99_us | 2674 | 227.8 | 4295 | 640.8 | 480.9 | 1350 | +60.6% | 3.37 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 1024 | 68.7 | 704.3 | 81.5 | 75.4 | 187.4 | -31.2% | 4.24 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 817.1 | 9.60 | 107.9 | 2.22 | 6.97 | 260.3 | -86.8% | 101.8 | **H3 better** |
| P+ vs H3 | pss_mib | 44.6 | 0.04 | 71.1 | 0.33 | 0.24 | 0.37 | +59.4% | 112.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -13537 | 934.9 | 13804 | 662.1 | 810.1 | 18557 | -202.0% | 33.8 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 150.8 | 26.3 | 594.5 | 80.8 | 60.1 | 63.7 | +294.3% | 7.39 | **N better** |
| N vs P+ | q1_p99_us | 420.0 | 77.4 | 2207 | 910.1 | 645.9 | 2540 | +425.6% | 2.77 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 10217 | 100.8 | 412.3 | 48.2 | 79.0 | 240.8 | -96.0% | 124.1 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 143.0 | 32.8 | 220.6 | 22.0 | 27.9 | 83.9 | +54.2% | 2.78 | BELOW FLOOR |
| N vs P+ | q4_p50_us | 2127 | 39.7 | 1301 | 48.4 | 44.3 | 51.3 | -38.8% | 18.6 | **P+ better** |
| N vs P+ | q5_p50_us | 2229 | 141.1 | 2520 | 551.4 | 402.4 | 141.0 | +13.1% | 0.73 | no difference |
| N vs P+ | q6_p50_us | 3698 | 75.0 | 3180 | 137.2 | 110.6 | 142.3 | -14.0% | 4.69 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 99.3 | 18.0 | 774.5 | 156.4 | 111.3 | 119.7 | +679.9% | 6.07 | **N better** |
| N vs P+ | c2_q1_p99_us | 926.7 | 682.0 | 5912 | 1844 | 1390 | 516.6 | +538.0% | 3.59 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 10120 | 210.2 | 640.3 | 154.3 | 184.4 | 2105 | -93.7% | 51.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 139.9 | 24.6 | 2109 | 206.3 | 146.9 | 37.9 | +1408.3% | 13.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 8204 | 1313 | 11343 | 3581 | 2697 | 1543 | +38.3% | 1.16 | no difference |
| N vs P+ | c4_q2_p50_us | 17708 | 2302 | 1751 | 219.0 | 1635 | 3856 | -90.1% | 9.76 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 39836 | 3139 | 10117 | 1521 | 2467 | 10568 | -74.6% | 12.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 565.2 | 34.9 | 830.7 | 79.2 | 61.2 | 147.0 | +47.0% | 4.34 | **N better** |
| N vs P+ | write_p99_us | 2005 | 426.9 | 2706 | 584.5 | 511.8 | 524.4 | +34.9% | 1.37 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1435 | 198.5 | 1049 | 153.7 | 177.5 | 150.8 | -26.9% | 2.17 | no difference |
| N vs P+ | view_state_bytes_per_key_read | 289.4 | 12.4 | 772.3 | 37.7 | 28.1 | 289.4 | +166.9% | 17.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 37.1 | 0.22 | 44.7 | 0.02 | 0.15 | 0.66 | +20.6% | 49.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 29241 | 1836 | -2056 | 186.3 | 1305 | 21952 | -107.0% | 24.0 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 150.8 | 26.3 | 78.5 | 23.3 | 24.8 | 1.82 | -47.9% | 2.91 | no difference |
| N vs P | q1_p99_us | 420.0 | 77.4 | 1004 | 236.5 | 175.9 | 79.6 | +139.0% | 3.32 | **N better** |
| N vs P | q2_p50_us | 10217 | 100.8 | 150.7 | 26.9 | 73.8 | 241.5 | -98.5% | 136.5 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 143.0 | 32.8 | 165.4 | 29.0 | 31.0 | 117.6 | +15.6% | 0.72 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q4_p50_us | 2127 | 39.7 | 216.9 | 17.1 | 30.6 | 64.7 | -89.8% | 62.4 | **P better** |
| N vs P | q5_p50_us | 2229 | 141.1 | 149.6 | 24.3 | 101.2 | 114.7 | -93.3% | 20.5 | **P better** |
| N vs P | q6_p50_us | 3698 | 75.0 | 590.2 | 56.6 | 66.5 | 27.9 | -84.0% | 46.8 | **P better** |
| N vs P | c2_q1_p50_us | 99.3 | 18.0 | 98.9 | 18.7 | 18.4 | 30.3 | -0.4% | 0.02 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 926.7 | 682.0 | 1712 | 106.6 | 488.1 | 1022 | +84.8% | 1.61 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 10120 | 210.2 | 174.1 | 11.7 | 148.9 | 2096 | -98.3% | 66.8 | **P better** |
| N vs P | c4_q1_p50_us | 139.9 | 24.6 | 140.8 | 28.0 | 26.3 | 75.4 | +0.6% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 8204 | 1313 | 2209 | 442.6 | 979.8 | 1145 | -73.1% | 6.12 | **P better** |
| N vs P | c4_q2_p50_us | 17708 | 2302 | 243.5 | 24.6 | 1628 | 3853 | -98.6% | 10.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 39836 | 3139 | 2495 | 593.0 | 2259 | 2121 | -93.7% | 16.5 | **P better** |
| N vs P | write_p50_us | 565.2 | 34.9 | 15207 | 237.2 | 169.5 | 66.9 | +2590.8% | 86.4 | **N better** |
| N vs P | write_p99_us | 2005 | 426.9 | 21658 | 1614 | 1181 | 732.6 | +980.1% | 16.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1435 | 198.5 | 61.3 | 1.48 | 140.4 | 76.5 | -95.7% | 9.79 | **N better** |
| N vs P | view_state_bytes_per_key_read | 289.4 | 12.4 | 262.3 | 17.6 | 15.2 | 231.6 | -9.4% | 1.78 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 37.1 | 0.22 | 45.0 | 0.10 | 0.17 | 0.64 | +21.4% | 47.1 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 29241 | 1836 | -1729 | 521.4 | 1350 | 21904 | -105.9% | 22.9 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 594.5 | 80.8 | 78.5 | 23.3 | 59.5 | 63.7 | -86.8% | 8.68 | **P better** |
| P+ vs P | q1_p99_us | 2207 | 910.1 | 1004 | 236.5 | 664.9 | 2541 | -54.5% | 1.81 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 412.3 | 48.2 | 150.7 | 26.9 | 39.0 | 20.4 | -63.4% | 6.70 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 220.6 | 22.0 | 165.4 | 29.0 | 25.8 | 132.4 | -25.0% | 2.14 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 1301 | 48.4 | 216.9 | 17.1 | 36.3 | 79.0 | -83.3% | 29.9 | **P better** |
| P+ vs P | q5_p50_us | 2520 | 551.4 | 149.6 | 24.3 | 390.3 | 82.1 | -94.1% | 6.07 | **P better** |
| P+ vs P | q6_p50_us | 3180 | 137.2 | 590.2 | 56.6 | 105.0 | 144.5 | -81.4% | 24.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 774.5 | 156.4 | 98.9 | 18.7 | 111.4 | 118.0 | -87.2% | 6.07 | **P better** |
| P+ vs P | c2_q1_p99_us | 5912 | 1844 | 1712 | 106.6 | 1306 | 1081 | -71.0% | 3.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 640.3 | 154.3 | 174.1 | 11.7 | 109.4 | 192.0 | -72.8% | 4.26 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p50_us | 2109 | 206.3 | 140.8 | 28.0 | 147.2 | 84.2 | -93.3% | 13.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 11343 | 3581 | 2209 | 442.6 | 2551 | 1172 | -80.5% | 3.58 | **P better** |
| P+ vs P | c4_q2_p50_us | 1751 | 219.0 | 243.5 | 24.6 | 155.8 | 191.5 | -86.1% | 9.68 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 10117 | 1521 | 2495 | 593.0 | 1155 | 10354 | -75.3% | 6.60 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 830.7 | 79.2 | 15207 | 237.2 | 176.8 | 155.3 | +1730.6% | 81.3 | **P+ better** |
| P+ vs P | write_p99_us | 2706 | 584.5 | 21658 | 1614 | 1214 | 674.4 | +700.4% | 15.6 | **P+ better** |
| P+ vs P | commits_per_s | 1049 | 153.7 | 61.3 | 1.48 | 108.7 | 130.0 | -94.2% | 9.09 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 772.3 | 37.7 | 262.3 | 17.6 | 29.4 | 281.5 | -66.0% | 17.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | pss_mib | 44.7 | 0.02 | 45.0 | 0.10 | 0.07 | 0.20 | +0.7% | 4.01 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2056 | 186.3 | -1729 | 521.4 | 391.5 | 2639 | -15.9% | 0.84 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 78.5 | 23.3 | 78.8 | 27.4 | 25.4 | 14.7 | +0.3% | 0.01 | BELOW FLOOR |
| P vs M | q1_p99_us | 1004 | 236.5 | 492.2 | 121.8 | 188.1 | 565.4 | -51.0% | 2.72 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 150.7 | 26.9 | 147.0 | 23.6 | 25.3 | 33.7 | -2.4% | 0.14 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 165.4 | 29.0 | 170.3 | 31.4 | 30.2 | 136.9 | +2.9% | 0.16 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 216.9 | 17.1 | 205.6 | 20.5 | 18.9 | 62.5 | -5.2% | 0.60 | BELOW FLOOR |
| P vs M | q5_p50_us | 149.6 | 24.3 | 100.7 | 27.3 | 25.9 | 11.1 | -32.7% | 1.89 | no difference |
| P vs M | q6_p50_us | 590.2 | 56.6 | 651.4 | 83.8 | 71.5 | 68.7 | +10.4% | 0.85 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 98.9 | 18.7 | 59.7 | 34.6 | 27.8 | 25.4 | -39.6% | 1.41 | no difference |
| P vs M | c2_q1_p99_us | 1712 | 106.6 | 1647 | 744.4 | 531.8 | 1026 | -3.8% | 0.12 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 174.1 | 11.7 | 158.6 | 67.1 | 48.2 | 18.8 | -8.9% | 0.32 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 140.8 | 28.0 | 173.1 | 29.7 | 28.9 | 127.5 | +23.0% | 1.12 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2209 | 442.6 | 2214 | 244.0 | 357.4 | 389.2 | +0.2% | 0.01 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 243.5 | 24.6 | 234.6 | 42.1 | 34.5 | 87.6 | -3.7% | 0.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 2495 | 593.0 | 2545 | 386.3 | 500.5 | 893.2 | +2.0% | 0.10 | BELOW FLOOR |
| P vs M | write_p50_us | 15207 | 237.2 | 15585 | 445.9 | 357.1 | 1018 | +2.5% | 1.06 | BELOW FLOOR |
| P vs M | write_p99_us | 21658 | 1614 | 22453 | 3259 | 2572 | 5086 | +3.7% | 0.31 | BELOW FLOOR |
| P vs M | commits_per_s | 61.3 | 1.48 | 60.7 | 4.05 | 3.05 | 7.42 | -1.0% | 0.20 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 262.3 | 17.6 | 262.3 | 17.6 | 17.6 | 221.7 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 45.0 | 0.10 | 44.7 | 0.06 | 0.08 | 0.06 | -0.7% | 4.01 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -1729 | 521.4 | -2360 | 250.6 | 409.1 | 2482 | +36.5% | 1.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 150.8 | 26.3 | 215.3 | 32.7 | 29.7 | 10.9 | +42.8% | 2.17 | no difference |
| N vs H3 | q1_p99_us | 420.0 | 77.4 | 599.1 | 219.4 | 164.5 | 309.3 | +42.7% | 1.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 10217 | 100.8 | 236.1 | 31.5 | 74.7 | 263.2 | -97.7% | 133.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 143.0 | 32.8 | 224.8 | 55.7 | 45.7 | 96.0 | +57.2% | 1.79 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q4_p50_us | 2127 | 39.7 | 1329 | 55.8 | 48.4 | 159.0 | -37.5% | 16.5 | **H3 better** |
| N vs H3 | q5_p50_us | 2229 | 141.1 | 2844 | 218.6 | 184.0 | 116.6 | +27.6% | 3.34 | **N better** |
| N vs H3 | q6_p50_us | 3698 | 75.0 | 3947 | 150.9 | 119.2 | 417.2 | +6.7% | 2.09 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 99.3 | 18.0 | 192.3 | 24.3 | 21.4 | 33.4 | +93.6% | 4.34 | **N better** |
| N vs H3 | c2_q1_p99_us | 926.7 | 682.0 | 788.5 | 125.6 | 490.3 | 330.6 | -14.9% | 0.28 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 10120 | 210.2 | 194.6 | 23.1 | 149.5 | 2096 | -98.1% | 66.4 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 139.9 | 24.6 | 317.2 | 71.5 | 53.5 | 23.8 | +126.8% | 3.32 | **N better** |
| N vs H3 | c4_q1_p99_us | 8204 | 1313 | 1536 | 500.1 | 993.6 | 1077 | -81.3% | 6.71 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 17708 | 2302 | 352.6 | 63.0 | 1628 | 3852 | -98.0% | 10.7 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 39836 | 3139 | 1440 | 480.3 | 2245 | 2120 | -96.4% | 17.1 | **H3 better** |
| N vs H3 | write_p50_us | 565.2 | 34.9 | 1125 | 112.1 | 83.0 | 36.0 | +99.1% | 6.75 | **N better** |
| N vs H3 | write_p99_us | 2005 | 426.9 | 4913 | 987.5 | 760.7 | 1353 | +145.0% | 3.82 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1435 | 198.5 | 644.1 | 110.6 | 160.7 | 79.8 | -55.1% | 4.92 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 289.4 | 12.4 | 108.5 | 1.03 | 8.82 | 170.5 | -62.5% | 20.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 37.1 | 0.22 | 72.4 | 0.59 | 0.44 | 0.91 | +95.1% | 79.8 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 29241 | 1836 | 14714 | 543.4 | 1354 | 21894 | -49.7% | 10.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H3 | q1_p50_us | 594.5 | 80.8 | 215.3 | 32.7 | 61.6 | 64.6 | -63.8% | 6.15 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2207 | 910.1 | 599.1 | 219.4 | 662.0 | 2558 | -72.9% | 2.43 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 412.3 | 48.2 | 236.1 | 31.5 | 40.7 | 106.5 | -42.7% | 4.33 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 220.6 | 22.0 | 224.8 | 55.7 | 42.3 | 113.7 | +1.9% | 0.10 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 1301 | 48.4 | 1329 | 55.8 | 52.2 | 165.4 | +2.1% | 0.52 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 2520 | 551.4 | 2844 | 218.6 | 419.4 | 84.7 | +12.8% | 0.77 | no difference |
| P+ vs H3 | q6_p50_us | 3180 | 137.2 | 3947 | 150.9 | 144.2 | 440.6 | +24.1% | 5.32 | **P+ better** |
| P+ vs H3 | c2_q1_p50_us | 774.5 | 156.4 | 192.3 | 24.3 | 111.9 | 118.9 | -75.2% | 5.20 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5912 | 1844 | 788.5 | 125.6 | 1307 | 482.2 | -86.7% | 3.92 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 640.3 | 154.3 | 194.6 | 23.1 | 110.3 | 191.5 | -69.6% | 4.04 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 2109 | 206.3 | 317.2 | 71.5 | 154.4 | 44.4 | -85.0% | 11.6 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 11343 | 3581 | 1536 | 500.1 | 2556 | 1106 | -86.5% | 3.84 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1751 | 219.0 | 352.6 | 63.0 | 161.1 | 171.5 | -79.9% | 8.68 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 10117 | 1521 | 1440 | 480.3 | 1128 | 10353 | -85.8% | 7.69 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 830.7 | 79.2 | 1125 | 112.1 | 97.0 | 144.8 | +35.5% | 3.04 | **P+ better** |
| P+ vs H3 | write_p99_us | 2706 | 584.5 | 4913 | 987.5 | 811.4 | 1323 | +81.5% | 2.72 | no difference |
| P+ vs H3 | commits_per_s | 1049 | 153.7 | 644.1 | 110.6 | 133.9 | 131.9 | -38.6% | 3.02 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 772.3 | 37.7 | 108.5 | 1.03 | 26.6 | 233.9 | -86.0% | 24.9 | **H3 better** |
| P+ vs H3 | pss_mib | 44.7 | 0.02 | 72.4 | 0.59 | 0.42 | 0.67 | +61.8% | 66.5 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2056 | 186.3 | 14714 | 543.4 | 406.2 | 2559 | -815.7% | 41.3 | REFUSED (warm-up MAD above 15% of the median on P+) |

## `multi`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 155.5 | 22.7 | 592.1 | 41.8 | 33.6 | 63.8 | +280.7% | 13.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 426.7 | 72.5 | 1632 | 188.4 | 142.8 | 1364 | +282.6% | 8.44 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 10148 | 96.9 | 430.9 | 26.6 | 71.1 | 2739 | -95.8% | 136.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 187.9 | 12.5 | 239.7 | 25.4 | 20.0 | 48.2 | +27.6% | 2.59 | no difference |
| N vs P+ | q4_p50_us | 2171 | 30.0 | 1328 | 21.5 | 26.1 | 229.7 | -38.8% | 32.3 | **P+ better** |
| N vs P+ | q5_p50_us | 1766 | 367.2 | 2589 | 447.9 | 409.5 | 507.7 | +46.6% | 2.01 | no difference |
| N vs P+ | q6_p50_us | 3775 | 72.7 | 3301 | 112.2 | 94.5 | 264.2 | -12.6% | 5.01 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 102.4 | 22.1 | 569.6 | 26.6 | 24.5 | 115.1 | +456.4% | 19.1 | **N better** |
| N vs P+ | c2_q1_p99_us | 435.2 | 162.4 | 5028 | 975.2 | 699.1 | 987.4 | +1055.4% | 6.57 | **N better** |
| N vs P+ | c2_q2_p50_us | 10441 | 256.8 | 476.8 | 20.3 | 182.2 | 122.3 | -95.4% | 54.7 | **P+ better** |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c4_q1_p50_us | 109.4 | 16.3 | 1717 | 88.1 | 63.3 | 68.8 | +1469.9% | 25.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 5965 | 578.2 | 10481 | 2515 | 1825 | 1333 | +75.7% | 2.47 | no difference |
| N vs P+ | c4_q2_p50_us | 18339 | 1256 | 1605 | 164.2 | 895.8 | 2095 | -91.2% | 18.7 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 37415 | 3933 | 7769 | 2143 | 3167 | 21985 | -79.2% | 9.36 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 589.7 | 26.9 | 848.8 | 43.4 | 36.1 | 33.3 | +43.9% | 7.17 | **N better** |
| N vs P+ | write_p99_us | 2440 | 1211 | 2627 | 113.0 | 860.2 | 1287 | +7.6% | 0.22 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1436 | 147.7 | 1003 | 67.6 | 114.8 | 329.8 | -30.1% | 3.77 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 285.2 | 8.12 | 765.0 | 34.6 | 25.1 | 379.1 | +168.3% | 19.1 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 37.0 | 0.46 | 44.8 | 0.02 | 0.32 | 0.98 | +21.0% | 23.9 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 28931 | 1668 | -2141 | 152.7 | 1184 | 22752 | -107.4% | 26.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 155.5 | 22.7 | 94.9 | 5.05 | 16.4 | 66.5 | -39.0% | 3.69 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q1_p99_us | 426.7 | 72.5 | 464.8 | 31.3 | 55.8 | 975.2 | +8.9% | 0.68 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 10148 | 96.9 | 174.6 | 13.7 | 69.2 | 2739 | -98.3% | 144.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 187.9 | 12.5 | 183.0 | 15.6 | 14.1 | 50.4 | -2.6% | 0.35 | BELOW FLOOR |
| N vs P | q4_p50_us | 2171 | 30.0 | 207.5 | 14.7 | 23.6 | 90.4 | -90.4% | 83.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q5_p50_us | 1766 | 367.2 | 128.8 | 13.2 | 259.8 | 502.2 | -92.7% | 6.30 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 3775 | 72.7 | 615.9 | 112.7 | 94.8 | 279.6 | -83.7% | 33.3 | **P better** |
| N vs P | c2_q1_p50_us | 102.4 | 22.1 | 94.7 | 14.3 | 18.6 | 23.0 | -7.5% | 0.41 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 435.2 | 162.4 | 1688 | 478.6 | 357.4 | 2383 | +287.8% | 3.50 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 10441 | 256.8 | 183.6 | 22.3 | 182.3 | 109.1 | -98.2% | 56.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c4_q1_p50_us | 109.4 | 16.3 | 134.1 | 42.0 | 31.8 | 49.4 | +22.6% | 0.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 5965 | 578.2 | 2046 | 199.4 | 432.5 | 1195 | -65.7% | 9.06 | **P better** |
| N vs P | c4_q2_p50_us | 18339 | 1256 | 240.0 | 55.2 | 889.1 | 2056 | -98.7% | 20.4 | **P better** |
| N vs P | c4_q2_p99_us | 37415 | 3933 | 3065 | 591.2 | 2812 | 21983 | -91.8% | 12.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 589.7 | 26.9 | 15473 | 266.5 | 189.4 | 243.5 | +2523.7% | 78.6 | **N better** |
| N vs P | write_p99_us | 2440 | 1211 | 22554 | 1666 | 1456 | 3787 | +824.2% | 13.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1436 | 147.7 | 62.2 | 3.42 | 104.5 | 323.9 | -95.7% | 13.2 | **N better** |
| N vs P | view_state_bytes_per_key_read | 285.2 | 8.12 | 253.2 | 16.7 | 13.1 | 231.4 | -11.2% | 2.43 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 37.0 | 0.46 | 44.6 | 0.03 | 0.33 | 0.97 | +20.6% | 23.5 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 28931 | 1668 | -2157 | 153.3 | 1184 | 22737 | -107.5% | 26.2 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 592.1 | 41.8 | 94.9 | 5.05 | 29.7 | 21.1 | -84.0% | 16.7 | **P better** |
| P+ vs P | q1_p99_us | 1632 | 188.4 | 464.8 | 31.3 | 135.1 | 1659 | -71.5% | 8.64 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 430.9 | 26.6 | 174.6 | 13.7 | 21.1 | 22.2 | -59.5% | 12.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 239.7 | 25.4 | 183.0 | 15.6 | 21.1 | 16.7 | -23.7% | 2.70 | no difference |
| P+ vs P | q4_p50_us | 1328 | 21.5 | 207.5 | 14.7 | 18.4 | 246.7 | -84.4% | 61.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q5_p50_us | 2589 | 447.9 | 128.8 | 13.2 | 316.8 | 96.3 | -95.0% | 7.76 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 3301 | 112.2 | 615.9 | 112.7 | 112.4 | 109.6 | -81.3% | 23.9 | **P better** |
| P+ vs P | c2_q1_p50_us | 569.6 | 26.6 | 94.7 | 14.3 | 21.4 | 115.1 | -83.4% | 22.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5028 | 975.2 | 1688 | 478.6 | 768.1 | 2580 | -66.4% | 4.35 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 476.8 | 20.3 | 183.6 | 22.3 | 21.3 | 94.4 | -61.5% | 13.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c4_q1_p50_us | 1717 | 88.1 | 134.1 | 42.0 | 69.0 | 51.1 | -92.2% | 22.9 | **P better** |
| P+ vs P | c4_q1_p99_us | 10481 | 2515 | 2046 | 199.4 | 1784 | 590.8 | -80.5% | 4.73 | **P better** |
| P+ vs P | c4_q2_p50_us | 1605 | 164.2 | 240.0 | 55.2 | 122.5 | 408.0 | -85.0% | 11.1 | **P better** |
| P+ vs P | c4_q2_p99_us | 7769 | 2143 | 3065 | 591.2 | 1572 | 375.4 | -60.5% | 2.99 | no difference |
| P+ vs P | write_p50_us | 848.8 | 43.4 | 15473 | 266.5 | 190.9 | 245.7 | +1723.0% | 76.6 | **P+ better** |
| P+ vs P | write_p99_us | 2627 | 113.0 | 22554 | 1666 | 1181 | 3891 | +758.6% | 16.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 1003 | 67.6 | 62.2 | 3.42 | 47.8 | 62.7 | -93.8% | 19.7 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 765.0 | 34.6 | 253.2 | 16.7 | 27.2 | 374.1 | -66.9% | 18.8 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.8 | 0.02 | 44.6 | 0.03 | 0.03 | 0.13 | -0.3% | 5.17 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2141 | 152.7 | -2157 | 153.3 | 153.0 | 2849 | +0.8% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 94.9 | 5.05 | 98.9 | 11.3 | 8.72 | 20.6 | +4.3% | 0.46 | BELOW FLOOR |
| P vs M | q1_p99_us | 464.8 | 31.3 | 480.8 | 57.1 | 46.0 | 1360 | +3.4% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 174.6 | 13.7 | 179.4 | 17.5 | 15.7 | 16.3 | +2.8% | 0.31 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 183.0 | 15.6 | 224.8 | 33.9 | 26.4 | 68.7 | +22.9% | 1.59 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 207.5 | 14.7 | 218.4 | 27.5 | 22.0 | 104.5 | +5.3% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q5_p50_us | 128.8 | 13.2 | 130.2 | 11.5 | 12.4 | 66.1 | +1.1% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | 615.9 | 112.7 | 624.9 | 45.2 | 85.8 | 119.7 | +1.5% | 0.11 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 94.7 | 14.3 | 101.8 | 19.9 | 17.3 | 27.2 | +7.4% | 0.41 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1688 | 478.6 | 1086 | 147.6 | 354.1 | 2498 | -35.6% | 1.70 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 183.6 | 22.3 | 193.5 | 22.5 | 22.4 | 62.5 | +5.4% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c4_q1_p50_us | 134.1 | 42.0 | 147.3 | 36.7 | 39.4 | 12.7 | +9.9% | 0.34 | no difference |
| P vs M | c4_q1_p99_us | 2046 | 199.4 | 2304 | 238.6 | 219.9 | 22.1 | +12.7% | 1.18 | no difference |
| P vs M | c4_q2_p50_us | 240.0 | 55.2 | 234.4 | 45.1 | 50.4 | 40.7 | -2.4% | 0.11 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3065 | 591.2 | 2019 | 443.9 | 522.8 | 799.9 | -34.1% | 2.00 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 15473 | 266.5 | 15667 | 640.4 | 490.5 | 2371 | +1.3% | 0.40 | BELOW FLOOR |
| P vs M | write_p99_us | 22554 | 1666 | 21822 | 2241 | 1974 | 3736 | -3.2% | 0.37 | BELOW FLOOR |
| P vs M | commits_per_s | 62.2 | 3.42 | 59.9 | 3.54 | 3.48 | 9.60 | -3.7% | 0.67 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 253.2 | 16.7 | 253.2 | 16.7 | 16.7 | 223.0 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.6 | 0.03 | 44.8 | 0.02 | 0.03 | 0.06 | +0.3% | 5.08 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2157 | 153.3 | -2020 | 143.0 | 148.2 | 2721 | -6.4% | 0.93 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 155.5 | 22.7 | 228.3 | 26.7 | 24.8 | 74.3 | +46.8% | 2.94 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 426.7 | 72.5 | 642.4 | 112.5 | 94.6 | 657.6 | +50.6% | 2.28 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 10148 | 96.9 | 234.1 | 27.2 | 71.2 | 2741 | -97.7% | 139.3 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 187.9 | 12.5 | 255.7 | 42.8 | 31.5 | 110.4 | +36.1% | 2.15 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q4_p50_us | 2171 | 30.0 | 1372 | 63.4 | 49.6 | 62.7 | -36.8% | 16.1 | **H3 better** |
| N vs H3 | q5_p50_us | 1766 | 367.2 | 2092 | 592.7 | 493.0 | 532.0 | +18.5% | 0.66 | BELOW FLOOR |
| N vs H3 | q6_p50_us | 3775 | 72.7 | 3734 | 49.3 | 62.2 | 364.6 | -1.1% | 0.67 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 102.4 | 22.1 | 209.3 | 16.1 | 19.3 | 157.6 | +104.5% | 5.54 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 435.2 | 162.4 | 1157 | 445.1 | 335.0 | 2391 | +165.8% | 2.15 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 10441 | 256.8 | 217.7 | 9.43 | 181.7 | 121.6 | -97.9% | 56.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c4_q1_p50_us | 109.4 | 16.3 | 329.2 | 42.7 | 32.3 | 146.8 | +201.0% | 6.80 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 5965 | 578.2 | 1696 | 422.3 | 506.3 | 6727 | -71.6% | 8.43 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 18339 | 1256 | 347.2 | 40.7 | 888.7 | 2056 | -98.1% | 20.2 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 37415 | 3933 | 2034 | 548.4 | 2808 | 22022 | -94.6% | 12.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 589.7 | 26.9 | 1148 | 46.1 | 37.8 | 107.6 | +94.7% | 14.8 | **N better** |
| N vs H3 | write_p99_us | 2440 | 1211 | 4875 | 1648 | 1446 | 734.8 | +99.8% | 1.68 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1436 | 147.7 | 665.4 | 89.0 | 121.9 | 328.6 | -53.7% | 6.32 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 285.2 | 8.12 | 109.3 | 0.48 | 5.75 | 169.5 | -61.7% | 30.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 37.0 | 0.46 | 71.1 | 0.19 | 0.35 | 0.98 | +92.3% | 97.4 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 28931 | 1668 | 13625 | 829.9 | 1317 | 25486 | -52.9% | 11.6 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 592.1 | 41.8 | 228.3 | 26.7 | 35.1 | 39.2 | -61.4% | 10.4 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1632 | 188.4 | 642.4 | 112.5 | 155.2 | 1495 | -60.6% | 6.38 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 430.9 | 26.6 | 234.1 | 27.2 | 26.9 | 109.8 | -45.7% | 7.31 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 239.7 | 25.4 | 255.7 | 42.8 | 35.2 | 99.6 | +6.6% | 0.45 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 1328 | 21.5 | 1372 | 63.4 | 47.3 | 238.0 | +3.3% | 0.92 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 2589 | 447.9 | 2092 | 592.7 | 525.3 | 200.3 | -19.2% | 0.95 | no difference |
| P+ vs H3 | q6_p50_us | 3301 | 112.2 | 3734 | 49.3 | 86.7 | 258.4 | +13.1% | 4.99 | **P+ better** |
| P+ vs H3 | c2_q1_p50_us | 569.6 | 26.6 | 209.3 | 16.1 | 22.0 | 193.8 | -63.3% | 16.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 5028 | 975.2 | 1157 | 445.1 | 758.0 | 2587 | -77.0% | 5.11 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 476.8 | 20.3 | 217.7 | 9.43 | 15.8 | 108.6 | -54.3% | 16.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c4_q1_p50_us | 1717 | 88.1 | 329.2 | 42.7 | 69.2 | 147.4 | -80.8% | 20.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 10481 | 2515 | 1696 | 422.3 | 1803 | 6646 | -83.8% | 4.87 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1605 | 164.2 | 347.2 | 40.7 | 119.7 | 406.7 | -78.4% | 10.5 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7769 | 2143 | 2034 | 548.4 | 1564 | 1368 | -73.8% | 3.67 | **H3 better** |
| P+ vs H3 | write_p50_us | 848.8 | 43.4 | 1148 | 46.1 | 44.8 | 112.5 | +35.3% | 6.69 | **P+ better** |
| P+ vs H3 | write_p99_us | 2627 | 113.0 | 4875 | 1648 | 1168 | 1157 | +85.6% | 1.93 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 1003 | 67.6 | 665.4 | 89.0 | 79.0 | 83.7 | -33.7% | 4.28 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 765.0 | 34.6 | 109.3 | 0.48 | 24.5 | 339.3 | -85.7% | 26.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.8 | 0.02 | 71.1 | 0.19 | 0.13 | 0.20 | +58.9% | 197.8 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2141 | 152.7 | 13625 | 829.9 | 596.7 | 11860 | -736.4% | 26.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 164.6 | 10.8 | 629.2 | 68.3 | 48.9 | 134.6 | +282.3% | 9.51 | **N better** |
| N vs P+ | q1_p99_us | 437.0 | 25.9 | 1789 | 586.5 | 415.1 | 696.7 | +309.5% | 3.26 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 10272 | 161.7 | 505.0 | 60.0 | 122.0 | 2286 | -95.1% | 80.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 187.3 | 20.4 | 266.6 | 32.9 | 27.4 | 57.3 | +42.3% | 2.90 | no difference |
| N vs P+ | q4_p50_us | 2279 | 142.3 | 1396 | 120.5 | 131.8 | 214.3 | -38.7% | 6.69 | **P+ better** |
| N vs P+ | q5_p50_us | 2294 | 19.5 | 2909 | 446.9 | 316.3 | 3438 | +26.8% | 1.95 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q6_p50_us | 3852 | 66.6 | 3245 | 127.6 | 101.8 | 580.1 | -15.8% | 5.97 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 88.9 | 6.70 | 695.8 | 117.7 | 83.3 | 51.4 | +682.7% | 7.28 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 547.5 | 182.0 | 5388 | 1306 | 932.4 | 1083 | +884.1% | 5.19 | **N better** |
| N vs P+ | c2_q2_p50_us | 10344 | 229.7 | 530.4 | 79.7 | 171.9 | 146.8 | -94.9% | 57.1 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 96.3 | 11.6 | 1861 | 219.0 | 155.1 | 376.3 | +1832.4% | 11.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 6569 | 1201 | 10522 | 4411 | 3232 | 1221 | +60.2% | 1.22 | no difference |
| N vs P+ | c4_q2_p50_us | 18220 | 719.9 | 1625 | 159.7 | 521.4 | 6131 | -91.1% | 31.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 31768 | 2583 | 8735 | 3088 | 2847 | 27886 | -72.5% | 8.09 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 630.3 | 15.5 | 905.1 | 86.9 | 62.4 | 15.2 | +43.6% | 4.40 | **N better** |
| N vs P+ | write_p99_us | 1649 | 735.1 | 3141 | 320.7 | 567.1 | 1857 | +90.5% | 2.63 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 1396 | 126.9 | 946.2 | 76.3 | 104.7 | 276.4 | -32.2% | 4.30 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 282.5 | 10.5 | 797.0 | 17.1 | 14.2 | 365.3 | +182.1% | 36.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 37.0 | 0.46 | 44.7 | 0.05 | 0.33 | 0.30 | +20.7% | 23.4 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 28531 | 1452 | -2351 | 286.8 | 1046 | 21723 | -108.2% | 29.5 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 164.6 | 10.8 | 95.8 | 9.57 | 10.2 | 19.3 | -41.8% | 6.74 | **P better** |
| N vs P | q1_p99_us | 437.0 | 25.9 | 513.3 | 66.2 | 50.2 | 166.3 | +17.5% | 1.52 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 10272 | 161.7 | 193.6 | 18.1 | 115.0 | 2286 | -98.1% | 87.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 187.3 | 20.4 | 235.7 | 41.9 | 32.9 | 78.1 | +25.8% | 1.47 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q4_p50_us | 2279 | 142.3 | 251.9 | 48.6 | 106.3 | 213.8 | -88.9% | 19.1 | **P better** |
| N vs P | q5_p50_us | 2294 | 19.5 | 121.1 | 26.8 | 23.4 | 818.7 | -94.7% | 92.7 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q6_p50_us | 3852 | 66.6 | 627.3 | 78.8 | 73.0 | 578.4 | -83.7% | 44.2 | **P better** |
| N vs P | c2_q1_p50_us | 88.9 | 6.70 | 74.4 | 20.3 | 15.1 | 39.7 | -16.3% | 0.96 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 547.5 | 182.0 | 2032 | 121.3 | 154.7 | 855.6 | +271.1% | 9.60 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 10344 | 229.7 | 135.8 | 27.3 | 163.6 | 147.1 | -98.7% | 62.4 | **P better** |
| N vs P | c4_q1_p50_us | 96.3 | 11.6 | 153.4 | 38.0 | 28.1 | 70.9 | +59.3% | 2.03 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 6569 | 1201 | 2782 | 376.1 | 889.8 | 1227 | -57.7% | 4.26 | **P better** |
| N vs P | c4_q2_p50_us | 18220 | 719.9 | 278.7 | 36.0 | 509.7 | 6131 | -98.5% | 35.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 31768 | 2583 | 3175 | 1082 | 1980 | 27786 | -90.0% | 14.4 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 630.3 | 15.5 | 15293 | 695.1 | 491.6 | 1530 | +2326.2% | 29.8 | **N better** |
| N vs P | write_p99_us | 1649 | 735.1 | 23019 | 3020 | 2198 | 1143 | +1296.2% | 9.72 | **N better** |
| N vs P | commits_per_s | 1396 | 126.9 | 61.3 | 4.77 | 89.8 | 126.5 | -95.6% | 14.9 | **N better** |
| N vs P | view_state_bytes_per_key_read | 282.5 | 10.5 | 248.8 | 14.0 | 12.4 | 230.7 | -11.9% | 2.72 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 37.0 | 0.46 | 44.8 | 0.02 | 0.32 | 0.36 | +20.8% | 23.7 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 28531 | 1452 | -2099 | 157.0 | 1032 | 21715 | -107.4% | 29.7 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 629.2 | 68.3 | 95.8 | 9.57 | 48.7 | 133.2 | -84.8% | 10.9 | **P better** |
| P+ vs P | q1_p99_us | 1789 | 586.5 | 513.3 | 66.2 | 417.3 | 714.4 | -71.3% | 3.06 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 505.0 | 60.0 | 193.6 | 18.1 | 44.3 | 10.2 | -61.7% | 7.02 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 266.6 | 32.9 | 235.7 | 41.9 | 37.7 | 95.8 | -11.6% | 0.82 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 1396 | 120.5 | 251.9 | 48.6 | 91.9 | 22.8 | -82.0% | 12.5 | **P better** |
| P+ vs P | q5_p50_us | 2909 | 446.9 | 121.1 | 26.8 | 316.6 | 3346 | -95.8% | 8.81 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q6_p50_us | 3245 | 127.6 | 627.3 | 78.8 | 106.1 | 45.2 | -80.7% | 24.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 695.8 | 117.7 | 74.4 | 20.3 | 84.4 | 40.8 | -89.3% | 7.36 | **P better** |
| P+ vs P | c2_q1_p99_us | 5388 | 1306 | 2032 | 121.3 | 927.4 | 1378 | -62.3% | 3.62 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 530.4 | 79.7 | 135.8 | 27.3 | 59.6 | 17.2 | -74.4% | 6.62 | **P better** |
| P+ vs P | c4_q1_p50_us | 1861 | 219.0 | 153.4 | 38.0 | 157.2 | 379.8 | -91.8% | 10.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 10522 | 4411 | 2782 | 376.1 | 3130 | 175.1 | -73.6% | 2.47 | no difference |
| P+ vs P | c4_q2_p50_us | 1625 | 159.7 | 278.7 | 36.0 | 115.8 | 68.0 | -82.8% | 11.6 | **P better** |
| P+ vs P | c4_q2_p99_us | 8735 | 3088 | 3175 | 1082 | 2314 | 3070 | -63.7% | 2.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 905.1 | 86.9 | 15293 | 695.1 | 495.3 | 1530 | +1589.6% | 29.0 | **P+ better** |
| P+ vs P | write_p99_us | 3141 | 320.7 | 23019 | 3020 | 2148 | 2178 | +632.9% | 9.26 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 946.2 | 76.3 | 61.3 | 4.77 | 54.0 | 246.2 | -93.5% | 16.4 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 797.0 | 17.1 | 248.8 | 14.0 | 15.6 | 362.1 | -68.8% | 35.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | pss_mib | 44.7 | 0.05 | 44.8 | 0.02 | 0.04 | 0.21 | +0.1% | 1.34 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2351 | 286.8 | -2099 | 157.0 | 231.2 | 3042 | -10.7% | 1.09 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 95.8 | 9.57 | 92.8 | 3.64 | 7.24 | 2.21 | -3.1% | 0.42 | no difference |
| P vs M | q1_p99_us | 513.3 | 66.2 | 512.1 | 95.6 | 82.2 | 219.6 | -0.2% | 0.01 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 193.6 | 18.1 | 188.0 | 12.0 | 15.4 | 8.86 | -2.9% | 0.36 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 235.7 | 41.9 | 200.6 | 14.4 | 31.3 | 130.7 | -14.9% | 1.12 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 251.9 | 48.6 | 208.2 | 23.1 | 38.1 | 122.9 | -17.3% | 1.15 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q5_p50_us | 121.1 | 26.8 | 121.9 | 7.58 | 19.7 | 287.3 | +0.7% | 0.04 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | 627.3 | 78.8 | 620.6 | 70.7 | 74.9 | 40.0 | -1.1% | 0.09 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 74.4 | 20.3 | 82.1 | 30.2 | 25.8 | 33.8 | +10.4% | 0.30 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 2032 | 121.3 | 1334 | 445.3 | 326.4 | 914.2 | -34.3% | 2.14 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 135.8 | 27.3 | 146.9 | 44.3 | 36.8 | 14.0 | +8.1% | 0.30 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 153.4 | 38.0 | 163.3 | 59.6 | 50.0 | 67.1 | +6.5% | 0.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2782 | 376.1 | 2167 | 487.3 | 435.2 | 505.0 | -22.1% | 1.41 | no difference |
| P vs M | c4_q2_p50_us | 278.7 | 36.0 | 269.4 | 97.0 | 73.2 | 45.8 | -3.3% | 0.13 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3175 | 1082 | 2419 | 388.5 | 812.6 | 1390 | -23.8% | 0.93 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 15293 | 695.1 | 15623 | 380.5 | 560.3 | 1994 | +2.2% | 0.59 | BELOW FLOOR |
| P vs M | write_p99_us | 23019 | 3020 | 23016 | 2956 | 2988 | 1166 | -0.0% | 0.00 | BELOW FLOOR |
| P vs M | commits_per_s | 61.3 | 4.77 | 59.8 | 1.95 | 3.64 | 12.3 | -2.4% | 0.40 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 248.8 | 14.0 | 248.8 | 14.0 | 14.0 | 225.5 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.8 | 0.02 | 44.8 | 0.02 | 0.02 | 0.20 | +0.1% | 2.79 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2099 | 157.0 | -2014 | 117.8 | 138.8 | 2737 | -4.1% | 0.61 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 164.6 | 10.8 | 254.3 | 42.0 | 30.6 | 57.3 | +54.5% | 2.93 | no difference |
| N vs H3 | q1_p99_us | 437.0 | 25.9 | 718.4 | 215.9 | 153.8 | 113.3 | +64.4% | 1.83 | no difference |
| N vs H3 | q2_p50_us | 10272 | 161.7 | 269.4 | 43.1 | 118.3 | 2286 | -97.4% | 84.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 187.3 | 20.4 | 278.4 | 34.1 | 28.1 | 40.6 | +48.6% | 3.24 | **N better** |
| N vs H3 | q4_p50_us | 2279 | 142.3 | 1405 | 69.4 | 112.0 | 273.9 | -38.4% | 7.81 | **H3 better** |
| N vs H3 | q5_p50_us | 2294 | 19.5 | 2932 | 150.0 | 106.9 | 1666 | +27.8% | 5.97 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q6_p50_us | 3852 | 66.6 | 4236 | 551.7 | 392.9 | 1262 | +10.0% | 0.98 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 88.9 | 6.70 | 229.4 | 29.2 | 21.2 | 120.6 | +158.0% | 6.63 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q1_p99_us | 547.5 | 182.0 | 883.6 | 216.2 | 199.8 | 3117 | +61.4% | 1.68 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 10344 | 229.7 | 209.0 | 23.3 | 163.3 | 174.4 | -98.0% | 62.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 96.3 | 11.6 | 342.8 | 16.1 | 14.0 | 49.5 | +256.0% | 17.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 6569 | 1201 | 1551 | 289.0 | 873.4 | 1226 | -76.4% | 5.74 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 18220 | 719.9 | 319.6 | 63.3 | 511.0 | 6131 | -98.2% | 35.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 31768 | 2583 | 1671 | 315.5 | 1840 | 27781 | -94.7% | 16.4 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 630.3 | 15.5 | 1244 | 115.3 | 82.3 | 24.5 | +97.4% | 7.46 | **N better** |
| N vs H3 | write_p99_us | 1649 | 735.1 | 5141 | 1411 | 1125 | 993.1 | +211.9% | 3.10 | **N better** |
| N vs H3 | commits_per_s | 1396 | 126.9 | 633.4 | 85.5 | 108.2 | 223.3 | -54.6% | 7.05 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 282.5 | 10.5 | 107.8 | 0.97 | 7.48 | 166.7 | -61.8% | 23.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 37.0 | 0.46 | 71.3 | 0.31 | 0.39 | 0.31 | +92.4% | 87.1 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 28531 | 1452 | 13726 | 361.1 | 1058 | 24373 | -51.9% | 14.0 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 629.2 | 68.3 | 254.3 | 42.0 | 56.7 | 143.7 | -59.6% | 6.62 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1789 | 586.5 | 718.4 | 215.9 | 441.9 | 704.0 | -59.8% | 2.42 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 505.0 | 60.0 | 269.4 | 43.1 | 52.2 | 15.0 | -46.7% | 4.51 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 266.6 | 32.9 | 278.4 | 34.1 | 33.5 | 68.7 | +4.4% | 0.35 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1396 | 120.5 | 1405 | 69.4 | 98.3 | 172.7 | +0.6% | 0.09 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 2909 | 446.9 | 2932 | 150.0 | 333.3 | 3647 | +0.8% | 0.07 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q6_p50_us | 3245 | 127.6 | 4236 | 551.7 | 400.4 | 1122 | +30.5% | 2.48 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 695.8 | 117.7 | 229.4 | 29.2 | 85.7 | 121.0 | -67.0% | 5.44 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 5388 | 1306 | 883.6 | 216.2 | 936.0 | 3299 | -83.6% | 4.81 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 530.4 | 79.7 | 209.0 | 23.3 | 58.7 | 95.3 | -60.6% | 5.48 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1861 | 219.0 | 342.8 | 16.1 | 155.3 | 376.4 | -81.6% | 9.78 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 10522 | 4411 | 1551 | 289.0 | 3126 | 167.6 | -85.3% | 2.87 | no difference |
| P+ vs H3 | c4_q2_p50_us | 1625 | 159.7 | 319.6 | 63.3 | 121.5 | 53.3 | -80.3% | 10.7 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8735 | 3088 | 1671 | 315.5 | 2195 | 3024 | -80.9% | 3.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 905.1 | 86.9 | 1244 | 115.3 | 102.1 | 21.5 | +37.5% | 3.32 | **P+ better** |
| P+ vs H3 | write_p99_us | 3141 | 320.7 | 5141 | 1411 | 1023 | 2103 | +63.7% | 1.96 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 946.2 | 76.3 | 633.4 | 85.5 | 81.0 | 307.4 | -33.1% | 3.86 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 797.0 | 17.1 | 107.8 | 0.97 | 12.1 | 325.1 | -86.5% | 56.9 | **H3 better** |
| P+ vs H3 | pss_mib | 44.7 | 0.05 | 71.3 | 0.31 | 0.22 | 0.12 | +59.4% | 118.1 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2351 | 286.8 | 13726 | 361.1 | 326.1 | 11480 | -683.7% | 49.3 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 160.6 | 14.0 | 629.9 | 50.3 | 36.9 | 191.6 | +292.3% | 12.7 | **N better** |
| N vs P+ | q1_p99_us | 398.2 | 33.3 | 1707 | 326.6 | 232.1 | 397.3 | +328.6% | 5.64 | **N better** |
| N vs P+ | q2_p50_us | 10279 | 143.3 | 481.4 | 46.8 | 106.6 | 386.0 | -95.3% | 91.9 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 178.8 | 28.2 | 248.4 | 22.3 | 25.4 | 14.1 | +39.0% | 2.74 | no difference |
| N vs P+ | q4_p50_us | 2168 | 42.3 | 1373 | 123.6 | 92.4 | 158.7 | -36.7% | 8.60 | **P+ better** |
| N vs P+ | q5_p50_us | 1860 | 338.9 | 2820 | 523.8 | 441.2 | 958.2 | +51.6% | 2.18 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 97.9 | 19.1 | 633.2 | 59.7 | 44.3 | 107.3 | +546.5% | 12.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 368.2 | 72.6 | 5835 | 1263 | 894.9 | 6380 | +1484.8% | 6.11 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 10266 | 269.1 | 553.5 | 27.4 | 191.3 | 653.5 | -94.6% | 50.8 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 106.7 | 11.0 | 1800 | 52.5 | 37.9 | 217.1 | +1586.9% | 44.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 5911 | 444.3 | 8947 | 2322 | 1671 | 15567 | +51.4% | 1.82 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 18228 | 590.9 | 1590 | 109.5 | 424.9 | 5578 | -91.3% | 39.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 35617 | 5963 | 8299 | 2301 | 4519 | 30050 | -76.7% | 6.04 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 603.1 | 19.6 | 882.0 | 104.2 | 75.0 | 118.9 | +46.2% | 3.72 | **N better** |
| N vs P+ | write_p99_us | 2122 | 975.0 | 2592 | 283.2 | 717.9 | 1370 | +22.2% | 0.66 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 1445 | 160.6 | 980.3 | 102.2 | 134.6 | 450.6 | -32.1% | 3.45 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 287.3 | 11.0 | 768.9 | 38.1 | 28.0 | 438.7 | +167.6% | 17.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 36.5 | 0.13 | 44.6 | 0.01 | 0.09 | 0.77 | +22.1% | 87.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 28598 | 1416 | -2221 | 131.0 | 1006 | 23682 | -107.8% | 30.6 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 160.6 | 14.0 | 88.3 | 5.50 | 10.6 | 56.4 | -45.0% | 6.82 | **P better** |
| N vs P | q1_p99_us | 398.2 | 33.3 | 459.4 | 48.1 | 41.4 | 821.0 | +15.4% | 1.48 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 10279 | 143.3 | 164.8 | 11.3 | 101.6 | 378.2 | -98.4% | 99.5 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 178.8 | 28.2 | 193.4 | 8.32 | 20.8 | 14.7 | +8.2% | 0.70 | BELOW FLOOR |
| N vs P | q4_p50_us | 2168 | 42.3 | 208.1 | 12.1 | 31.1 | 9.66 | -90.4% | 63.0 | **P better** |
| N vs P | q5_p50_us | 1860 | 338.9 | 108.0 | 12.3 | 239.8 | 743.4 | -94.2% | 7.31 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 97.9 | 19.1 | 108.7 | 28.0 | 23.9 | 51.1 | +11.0% | 0.45 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 368.2 | 72.6 | 2018 | 341.1 | 246.6 | 1450 | +448.2% | 6.69 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 10266 | 269.1 | 203.1 | 36.9 | 192.1 | 644.8 | -98.0% | 52.4 | **P better** |
| N vs P | c4_q1_p50_us | 106.7 | 11.0 | 124.3 | 21.3 | 17.0 | 49.3 | +16.5% | 1.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 5911 | 444.3 | 2412 | 339.5 | 395.4 | 1753 | -59.2% | 8.85 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 18228 | 590.9 | 257.3 | 51.5 | 419.4 | 5578 | -98.6% | 42.8 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 35617 | 5963 | 2433 | 499.5 | 4231 | 1499 | -93.2% | 7.84 | **P better** |
| N vs P | write_p50_us | 603.1 | 19.6 | 15265 | 444.4 | 314.5 | 206.2 | +2430.9% | 46.6 | **N better** |
| N vs P | write_p99_us | 2122 | 975.0 | 25277 | 3112 | 2306 | 608.7 | +1091.4% | 10.0 | **N better** |
| N vs P | commits_per_s | 1445 | 160.6 | 62.3 | 2.83 | 113.6 | 441.6 | -95.7% | 12.2 | **N better** |
| N vs P | view_state_bytes_per_key_read | 287.3 | 11.0 | 253.9 | 16.9 | 14.2 | 228.9 | -11.6% | 2.34 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 36.5 | 0.13 | 44.7 | 0.01 | 0.09 | 0.76 | +22.5% | 88.5 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 28598 | 1416 | -1990 | 105.7 | 1004 | 23681 | -107.0% | 30.5 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 629.9 | 50.3 | 88.3 | 5.50 | 35.8 | 184.3 | -86.0% | 15.1 | **P better** |
| P+ vs P | q1_p99_us | 1707 | 326.6 | 459.4 | 48.1 | 233.4 | 906.5 | -73.1% | 5.34 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 481.4 | 46.8 | 164.8 | 11.3 | 34.0 | 97.7 | -65.8% | 9.30 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 248.4 | 22.3 | 193.4 | 8.32 | 16.9 | 18.2 | -22.2% | 3.27 | **P better** |
| P+ vs P | q4_p50_us | 1373 | 123.6 | 208.1 | 12.1 | 87.8 | 158.5 | -84.8% | 13.3 | **P better** |
| P+ vs P | q5_p50_us | 2820 | 523.8 | 108.0 | 12.3 | 370.5 | 605.1 | -96.2% | 7.32 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 633.2 | 59.7 | 108.7 | 28.0 | 46.6 | 98.8 | -82.8% | 11.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5835 | 1263 | 2018 | 341.1 | 925.4 | 6401 | -65.4% | 4.12 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 553.5 | 27.4 | 203.1 | 36.9 | 32.5 | 107.8 | -63.3% | 10.8 | **P better** |
| P+ vs P | c4_q1_p50_us | 1800 | 52.5 | 124.3 | 21.3 | 40.0 | 215.3 | -93.1% | 41.8 | **P better** |
| P+ vs P | c4_q1_p99_us | 8947 | 2322 | 2412 | 339.5 | 1659 | 15495 | -73.0% | 3.94 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1590 | 109.5 | 257.3 | 51.5 | 85.6 | 75.5 | -83.8% | 15.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 8299 | 2301 | 2433 | 499.5 | 1665 | 30028 | -70.7% | 3.52 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 882.0 | 104.2 | 15265 | 444.4 | 322.8 | 184.9 | +1630.8% | 44.6 | **P+ better** |
| P+ vs P | write_p99_us | 2592 | 283.2 | 25277 | 3112 | 2209 | 1499 | +875.2% | 10.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 980.3 | 102.2 | 62.3 | 2.83 | 72.3 | 89.3 | -93.6% | 12.7 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 768.9 | 38.1 | 253.9 | 16.9 | 29.5 | 426.6 | -67.0% | 17.5 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.6 | 0.01 | 44.7 | 0.01 | 0.01 | 0.08 | +0.3% | 12.4 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2221 | 131.0 | -1990 | 105.7 | 119.0 | 2713 | -10.4% | 1.95 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 88.3 | 5.50 | 92.4 | 8.48 | 7.15 | 37.9 | +4.7% | 0.58 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q1_p99_us | 459.4 | 48.1 | 476.0 | 72.0 | 61.2 | 931.2 | +3.6% | 0.27 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 164.8 | 11.3 | 177.0 | 17.4 | 14.7 | 45.5 | +7.4% | 0.84 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 193.4 | 8.32 | 213.7 | 39.8 | 28.7 | 20.8 | +10.5% | 0.71 | BELOW FLOOR |
| P vs M | q4_p50_us | 208.1 | 12.1 | 210.8 | 24.7 | 19.4 | 23.5 | +1.3% | 0.14 | BELOW FLOOR |
| P vs M | q5_p50_us | 108.0 | 12.3 | 133.8 | 5.77 | 9.61 | 32.4 | +23.9% | 2.69 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 108.7 | 28.0 | 82.5 | 21.3 | 24.8 | 21.2 | -24.2% | 1.06 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 2018 | 341.1 | 1391 | 581.7 | 476.9 | 1090 | -31.1% | 1.32 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 203.1 | 36.9 | 154.6 | 34.9 | 35.9 | 31.1 | -23.9% | 1.35 | no difference |
| P vs M | c4_q1_p50_us | 124.3 | 21.3 | 118.3 | 25.3 | 23.4 | 48.7 | -4.8% | 0.25 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2412 | 339.5 | 2255 | 246.4 | 296.6 | 1298 | -6.5% | 0.53 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p50_us | 257.3 | 51.5 | 232.4 | 40.7 | 46.4 | 107.9 | -9.7% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p99_us | 2433 | 499.5 | 2228 | 277.8 | 404.1 | 803.2 | -8.4% | 0.51 | BELOW FLOOR |
| P vs M | write_p50_us | 15265 | 444.4 | 15434 | 388.4 | 417.4 | 243.1 | +1.1% | 0.40 | BELOW FLOOR |
| P vs M | write_p99_us | 25277 | 3112 | 22337 | 1959 | 2600 | 1299 | -11.6% | 1.13 | no difference |
| P vs M | commits_per_s | 62.3 | 2.83 | 61.0 | 3.01 | 2.92 | 4.79 | -2.0% | 0.44 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 253.9 | 16.9 | 253.9 | 16.9 | 16.9 | 204.6 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.7 | 0.01 | 44.9 | 0.02 | 0.02 | 0.08 | +0.5% | 13.5 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -1990 | 105.7 | -2110 | 182.9 | 149.4 | 2617 | +6.0% | 0.80 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 160.6 | 14.0 | 208.0 | 31.0 | 24.0 | 56.2 | +29.5% | 1.97 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 398.2 | 33.3 | 584.3 | 85.9 | 65.2 | 236.7 | +46.8% | 2.86 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 10279 | 143.3 | 226.3 | 19.8 | 102.3 | 379.2 | -97.8% | 98.3 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 178.8 | 28.2 | 252.1 | 44.5 | 37.3 | 6.56 | +41.0% | 1.97 | no difference |
| N vs H3 | q4_p50_us | 2168 | 42.3 | 1399 | 72.1 | 59.1 | 731.1 | -35.5% | 13.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q5_p50_us | 1860 | 338.9 | 2179 | 660.8 | 525.1 | 822.6 | +17.1% | 0.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 97.9 | 19.1 | 174.4 | 23.8 | 21.6 | 49.0 | +78.0% | 3.54 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 368.2 | 72.6 | 1015 | 335.0 | 242.4 | 989.7 | +175.7% | 2.67 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 10266 | 269.1 | 183.5 | 6.09 | 190.3 | 646.1 | -98.2% | 53.0 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 106.7 | 11.0 | 324.8 | 41.4 | 30.3 | 121.8 | +204.4% | 7.20 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 5911 | 444.3 | 1089 | 86.4 | 320.1 | 2191 | -81.6% | 15.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 18228 | 590.9 | 343.0 | 15.8 | 418.0 | 5580 | -98.1% | 42.8 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p99_us | 35617 | 5963 | 1161 | 165.4 | 4218 | 1424 | -96.7% | 8.17 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 603.1 | 19.6 | 1102 | 62.7 | 46.5 | 113.5 | +82.7% | 10.7 | **N better** |
| N vs H3 | write_p99_us | 2122 | 975.0 | 4402 | 1296 | 1147 | 7982 | +107.5% | 1.99 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | commits_per_s | 1445 | 160.6 | 681.8 | 108.2 | 136.9 | 451.3 | -52.8% | 5.57 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 287.3 | 11.0 | 108.9 | 0.62 | 7.76 | 177.6 | -62.1% | 23.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 36.5 | 0.13 | 71.3 | 0.20 | 0.17 | 0.79 | +95.3% | 204.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 28598 | 1416 | 13669 | 874.6 | 1177 | 26742 | -52.2% | 12.7 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 629.9 | 50.3 | 208.0 | 31.0 | 41.8 | 184.2 | -67.0% | 10.1 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1707 | 326.6 | 584.3 | 85.9 | 238.8 | 451.3 | -65.8% | 4.70 | **H3 better** |
| P+ vs H3 | q2_p50_us | 481.4 | 46.8 | 226.3 | 19.8 | 35.9 | 101.5 | -53.0% | 7.10 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 248.4 | 22.3 | 252.1 | 44.5 | 35.2 | 12.5 | +1.5% | 0.11 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1373 | 123.6 | 1399 | 72.1 | 101.2 | 748.1 | +1.9% | 0.26 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q5_p50_us | 2820 | 523.8 | 2179 | 660.8 | 596.3 | 700.1 | -22.7% | 1.07 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 633.2 | 59.7 | 174.4 | 23.8 | 45.5 | 97.7 | -72.5% | 10.1 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5835 | 1263 | 1015 | 335.0 | 924.3 | 6312 | -82.6% | 5.21 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 553.5 | 27.4 | 183.5 | 6.09 | 19.8 | 115.5 | -66.9% | 18.6 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1800 | 52.5 | 324.8 | 41.4 | 47.3 | 242.4 | -82.0% | 31.2 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 8947 | 2322 | 1089 | 86.4 | 1643 | 15550 | -87.8% | 4.78 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1590 | 109.5 | 343.0 | 15.8 | 78.2 | 172.2 | -78.4% | 15.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 8299 | 2301 | 1161 | 165.4 | 1631 | 30025 | -86.0% | 4.38 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 882.0 | 104.2 | 1102 | 62.7 | 86.0 | 67.4 | +24.9% | 2.56 | no difference |
| P+ vs H3 | write_p99_us | 2592 | 283.2 | 4402 | 1296 | 938.0 | 8099 | +69.8% | 1.93 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | commits_per_s | 980.3 | 102.2 | 681.8 | 108.2 | 105.2 | 128.8 | -30.5% | 2.84 | no difference |
| P+ vs H3 | view_state_bytes_per_key_read | 768.9 | 38.1 | 108.9 | 0.62 | 26.9 | 401.4 | -85.8% | 24.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.6 | 0.01 | 71.3 | 0.20 | 0.14 | 0.21 | +59.9% | 186.6 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2221 | 131.0 | 13669 | 874.6 | 625.4 | 12717 | -715.3% | 25.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 327.9 | 33.8 | 635.9 | 22.4 | 28.7 | 60.6 | +93.9% | 10.7 | **N better** |
| N vs P+ | q1_p99_us | 893.9 | 159.8 | 1488 | 147.1 | 153.6 | 448.5 | +66.5% | 3.87 | **N better** |
| N vs P+ | q2_p50_us | 87150 | 9862 | 494.7 | 20.8 | 6973 | 5353 | -99.4% | 12.4 | **P+ better** |
| N vs P+ | q3_p50_us | 214.6 | 32.7 | 289.7 | 40.9 | 37.0 | 41.7 | +35.0% | 2.03 | no difference |
| N vs P+ | q4_p50_us | 20094 | 330.7 | 11008 | 430.4 | 383.8 | 1201 | -45.2% | 23.7 | **P+ better** |
| N vs P+ | q5_p50_us | 15821 | 2749 | 15673 | 3166 | 2965 | 522.2 | -0.9% | 0.05 | BELOW FLOOR |
| N vs P+ | q6_p50_us | 53535 | 1368 | 41584 | 1417 | 1393 | 3179 | -22.3% | 8.58 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 216.8 | 25.0 | 634.9 | 69.6 | 52.3 | 79.0 | +192.8% | 8.00 | **N better** |
| N vs P+ | c2_q1_p99_us | 4303 | 677.4 | 4845 | 749.5 | 714.4 | 1095 | +12.6% | 0.76 | BELOW FLOOR |
| N vs P+ | c2_q2_p50_us | 96171 | 13911 | 549.5 | 83.4 | 9837 | 9593 | -99.4% | 9.72 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 243.0 | 10.3 | 1788 | 127.1 | 90.1 | 355.8 | +635.9% | 17.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 8886 | 567.8 | 6976 | 1516 | 1145 | 5838 | -21.5% | 1.67 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 162622 | 13631 | 1644 | 155.2 | 9639 | 3043 | -99.0% | 16.7 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 331933 | 16785 | 6576 | 895.3 | 11886 | 56166 | -98.0% | 27.4 | **P+ better** |
| N vs P+ | write_p50_us | 751.7 | 15.8 | 818.5 | 64.7 | 47.1 | 161.0 | +8.9% | 1.42 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1758 | 490.6 | 2725 | 238.2 | 385.6 | 6405 | +55.0% | 2.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1187 | 34.4 | 1008 | 69.4 | 54.8 | 364.7 | -15.1% | 3.27 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 648.0 | 119.9 | 825.3 | 137.6 | 129.1 | 2042 | +27.4% | 1.37 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 308.7 | 0.22 | 44.8 | 0.03 | 0.16 | 1.46 | -85.5% | 1657 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 93359 | 19511 | -773.9 | 173.6 | 13797 | 208680 | -100.8% | 6.82 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 327.9 | 33.8 | 103.7 | 5.12 | 24.2 | 4.59 | -68.4% | 9.26 | **P better** |
| N vs P | q1_p99_us | 893.9 | 159.8 | 521.2 | 61.4 | 121.0 | 76.7 | -41.7% | 3.08 | **P better** |
| N vs P | q2_p50_us | 87150 | 9862 | 200.7 | 23.3 | 6973 | 5353 | -99.8% | 12.5 | **P better** |
| N vs P | q3_p50_us | 214.6 | 32.7 | 246.1 | 39.8 | 36.4 | 50.3 | +14.7% | 0.86 | BELOW FLOOR |
| N vs P | q4_p50_us | 20094 | 330.7 | 921.9 | 32.9 | 235.0 | 1033 | -95.4% | 81.6 | **P better** |
| N vs P | q5_p50_us | 15821 | 2749 | 164.2 | 36.0 | 1944 | 499.1 | -99.0% | 8.06 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 53535 | 1368 | 3358 | 137.9 | 972.0 | 3194 | -93.7% | 51.6 | **P better** |
| N vs P | c2_q1_p50_us | 216.8 | 25.0 | 89.8 | 31.0 | 28.2 | 43.3 | -58.6% | 4.51 | **P better** |
| N vs P | c2_q1_p99_us | 4303 | 677.4 | 1926 | 472.6 | 584.0 | 1255 | -55.2% | 4.07 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 96171 | 13911 | 211.9 | 38.8 | 9836 | 9593 | -99.8% | 9.76 | **P better** |
| N vs P | c4_q1_p50_us | 243.0 | 10.3 | 119.9 | 36.7 | 27.0 | 104.4 | -50.6% | 4.56 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 8886 | 567.8 | 2620 | 155.6 | 416.3 | 813.0 | -70.5% | 15.1 | **P better** |
| N vs P | c4_q2_p50_us | 162622 | 13631 | 238.1 | 59.1 | 9638 | 3019 | -99.9% | 16.8 | **P better** |
| N vs P | c4_q2_p99_us | 331933 | 16785 | 4289 | 617.0 | 11877 | 56151 | -98.7% | 27.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 751.7 | 15.8 | 148921 | 2548 | 1801 | 1704 | +19710.5% | 82.3 | **N better** |
| N vs P | write_p99_us | 1758 | 490.6 | 187211 | 23096 | 16335 | 31180 | +10546.6% | 11.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1187 | 34.4 | 6.56 | 0.21 | 24.3 | 360.2 | -99.4% | 48.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | view_state_bytes_per_key_read | 648.0 | 119.9 | 509.7 | 104.5 | 112.5 | 1735 | -21.3% | 1.23 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 308.7 | 0.22 | 44.9 | 0.03 | 0.16 | 1.46 | -85.5% | 1663 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 93359 | 19511 | -684.5 | 154.5 | 13797 | 208680 | -100.7% | 6.82 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 635.9 | 22.4 | 103.7 | 5.12 | 16.2 | 60.7 | -83.7% | 32.8 | **P better** |
| P+ vs P | q1_p99_us | 1488 | 147.1 | 521.2 | 61.4 | 112.7 | 445.7 | -65.0% | 8.58 | **P better** |
| P+ vs P | q2_p50_us | 494.7 | 20.8 | 200.7 | 23.3 | 22.0 | 46.9 | -59.4% | 13.3 | **P better** |
| P+ vs P | q3_p50_us | 289.7 | 40.9 | 246.1 | 39.8 | 40.3 | 33.6 | -15.1% | 1.08 | no difference |
| P+ vs P | q4_p50_us | 11008 | 430.4 | 921.9 | 32.9 | 305.2 | 612.1 | -91.6% | 33.0 | **P better** |
| P+ vs P | q5_p50_us | 15673 | 3166 | 164.2 | 36.0 | 2239 | 174.3 | -99.0% | 6.93 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 41584 | 1417 | 3358 | 137.9 | 1007 | 701.5 | -91.9% | 38.0 | **P better** |
| P+ vs P | c2_q1_p50_us | 634.9 | 69.6 | 89.8 | 31.0 | 53.9 | 68.7 | -85.9% | 10.1 | **P better** |
| P+ vs P | c2_q1_p99_us | 4845 | 749.5 | 1926 | 472.6 | 626.5 | 1139 | -60.2% | 4.66 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 549.5 | 83.4 | 211.9 | 38.8 | 65.0 | 65.0 | -61.4% | 5.19 | **P better** |
| P+ vs P | c4_q1_p50_us | 1788 | 127.1 | 119.9 | 36.7 | 93.5 | 365.0 | -93.3% | 17.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 6976 | 1516 | 2620 | 155.6 | 1077 | 5806 | -62.4% | 4.04 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1644 | 155.2 | 238.1 | 59.1 | 117.5 | 380.7 | -85.5% | 12.0 | **P better** |
| P+ vs P | c4_q2_p99_us | 6576 | 895.3 | 4289 | 617.0 | 768.8 | 2722 | -34.8% | 2.98 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 818.5 | 64.7 | 148921 | 2548 | 1802 | 1711 | +18095.1% | 82.2 | **P+ better** |
| P+ vs P | write_p99_us | 2725 | 238.2 | 187211 | 23096 | 16332 | 30517 | +6770.5% | 11.3 | **P+ better** |
| P+ vs P | commits_per_s | 1008 | 69.4 | 6.56 | 0.21 | 49.1 | 57.3 | -99.3% | 20.4 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 825.3 | 137.6 | 509.7 | 104.5 | 122.2 | 1950 | -38.2% | 2.58 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.8 | 0.03 | 44.9 | 0.03 | 0.03 | 0.19 | +0.3% | 4.53 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -773.9 | 173.6 | -684.5 | 154.5 | 164.3 | 2412 | -11.6% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 103.7 | 5.12 | 102.2 | 6.99 | 6.13 | 6.55 | -1.4% | 0.24 | BELOW FLOOR |
| P vs M | q1_p99_us | 521.2 | 61.4 | 679.6 | 169.2 | 127.3 | 45.9 | +30.4% | 1.24 | no difference |
| P vs M | q2_p50_us | 200.7 | 23.3 | 182.0 | 15.5 | 19.8 | 21.6 | -9.3% | 0.94 | BELOW FLOOR |
| P vs M | q3_p50_us | 246.1 | 39.8 | 248.7 | 36.6 | 38.2 | 43.4 | +1.1% | 0.07 | BELOW FLOOR |
| P vs M | q4_p50_us | 921.9 | 32.9 | 906.2 | 59.7 | 48.2 | 178.4 | -1.7% | 0.33 | BELOW FLOOR |
| P vs M | q5_p50_us | 164.2 | 36.0 | 135.8 | 18.0 | 28.5 | 60.1 | -17.3% | 1.00 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 3358 | 137.9 | 3207 | 124.5 | 131.4 | 585.6 | -4.5% | 1.14 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 89.8 | 31.0 | 83.4 | 10.7 | 23.2 | 17.9 | -7.2% | 0.28 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1926 | 472.6 | 1373 | 320.9 | 403.9 | 924.6 | -28.7% | 1.37 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 211.9 | 38.8 | 183.2 | 26.3 | 33.1 | 29.3 | -13.5% | 0.87 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 119.9 | 36.7 | 132.1 | 38.0 | 37.3 | 104.4 | +10.1% | 0.33 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2620 | 155.6 | 2354 | 531.1 | 391.3 | 389.0 | -10.2% | 0.68 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 238.1 | 59.1 | 256.8 | 49.4 | 54.5 | 43.9 | +7.9% | 0.34 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 4289 | 617.0 | 3571 | 908.2 | 776.4 | 2208 | -16.7% | 0.92 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 148921 | 2548 | 145044 | 2753 | 2652 | 1807 | -2.6% | 1.46 | no difference |
| P vs M | write_p99_us | 187211 | 23096 | 189458 | 15504 | 19670 | 31613 | +1.2% | 0.11 | BELOW FLOOR |
| P vs M | commits_per_s | 6.56 | 0.21 | 6.81 | 0.20 | 0.21 | 0.25 | +3.8% | 1.21 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 509.7 | 104.5 | 509.7 | 104.5 | 104.5 | 1612 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.9 | 0.03 | 45.1 | 0.03 | 0.03 | 0.15 | +0.3% | 4.71 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -684.5 | 154.5 | -747.8 | 164.9 | 159.8 | 2453 | +9.3% | 0.40 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 327.9 | 33.8 | 315.7 | 27.5 | 30.9 | 22.8 | -3.7% | 0.39 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 893.9 | 159.8 | 1007 | 234.5 | 200.6 | 133.5 | +12.7% | 0.56 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 87150 | 9862 | 312.3 | 29.0 | 6973 | 5353 | -99.6% | 12.5 | **H3 better** |
| N vs H3 | q3_p50_us | 214.6 | 32.7 | 266.3 | 15.2 | 25.5 | 80.8 | +24.1% | 2.03 | BELOW FLOOR |
| N vs H3 | q4_p50_us | 20094 | 330.7 | 11138 | 130.4 | 251.4 | 2462 | -44.6% | 35.6 | **H3 better** |
| N vs H3 | q5_p50_us | 15821 | 2749 | 17146 | 5055 | 4068 | 4002 | +8.4% | 0.33 | BELOW FLOOR |
| N vs H3 | q6_p50_us | 53535 | 1368 | 44811 | 2084 | 1763 | 3254 | -16.3% | 4.95 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 216.8 | 25.0 | 370.4 | 67.8 | 51.1 | 191.8 | +70.9% | 3.01 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 4303 | 677.4 | 1588 | 493.2 | 592.5 | 1033 | -63.1% | 4.58 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 96171 | 13911 | 424.7 | 46.5 | 9836 | 9594 | -99.6% | 9.73 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 243.0 | 10.3 | 601.0 | 85.6 | 61.0 | 46.3 | +147.3% | 5.87 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 8886 | 567.8 | 2856 | 895.4 | 749.7 | 2139 | -67.9% | 8.04 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 162622 | 13631 | 608.7 | 91.3 | 9639 | 3019 | -99.6% | 16.8 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 331933 | 16785 | 2876 | 766.2 | 11881 | 56128 | -99.1% | 27.7 | **H3 better** |
| N vs H3 | write_p50_us | 751.7 | 15.8 | 1231 | 38.9 | 29.7 | 72.2 | +63.7% | 16.1 | **N better** |
| N vs H3 | write_p99_us | 1758 | 490.6 | 4615 | 727.6 | 620.5 | 6430 | +162.4% | 4.60 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1187 | 34.4 | 649.0 | 74.5 | 58.0 | 364.0 | -45.3% | 9.26 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 648.0 | 119.9 | 122.2 | 3.46 | 84.8 | 1301 | -81.1% | 6.20 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 308.7 | 0.22 | 110.3 | 0.11 | 0.18 | 1.53 | -64.3% | 1123 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 93359 | 19511 | 19035 | 3989 | 14082 | 212644 | -79.6% | 5.28 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 635.9 | 22.4 | 315.7 | 27.5 | 25.1 | 64.6 | -50.3% | 12.8 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1488 | 147.1 | 1007 | 234.5 | 195.7 | 458.9 | -32.3% | 2.46 | no difference |
| P+ vs H3 | q2_p50_us | 494.7 | 20.8 | 312.3 | 29.0 | 25.2 | 51.3 | -36.9% | 7.23 | **H3 better** |
| P+ vs H3 | q3_p50_us | 289.7 | 40.9 | 266.3 | 15.2 | 30.9 | 71.6 | -8.1% | 0.76 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 11008 | 430.4 | 11138 | 130.4 | 318.0 | 2317 | +1.2% | 0.41 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 15673 | 3166 | 17146 | 5055 | 4217 | 3975 | +9.4% | 0.35 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 41584 | 1417 | 44811 | 2084 | 1782 | 938.8 | +7.8% | 1.81 | no difference |
| P+ vs H3 | c2_q1_p50_us | 634.9 | 69.6 | 370.4 | 67.8 | 68.7 | 199.0 | -41.6% | 3.85 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 4845 | 749.5 | 1588 | 493.2 | 634.4 | 889.0 | -67.2% | 5.13 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 549.5 | 83.4 | 424.7 | 46.5 | 67.5 | 123.3 | -22.7% | 1.85 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1788 | 127.1 | 601.0 | 85.6 | 108.4 | 352.8 | -66.4% | 11.0 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6976 | 1516 | 2856 | 895.4 | 1245 | 6134 | -59.1% | 3.31 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1644 | 155.2 | 608.7 | 91.3 | 127.4 | 384.0 | -63.0% | 8.13 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6576 | 895.3 | 2876 | 766.2 | 833.3 | 2200 | -56.3% | 4.44 | **H3 better** |
| P+ vs H3 | write_p50_us | 818.5 | 64.7 | 1231 | 38.9 | 53.4 | 162.9 | +50.4% | 7.72 | **P+ better** |
| P+ vs H3 | write_p99_us | 2725 | 238.2 | 4615 | 727.6 | 541.3 | 679.0 | +69.4% | 3.49 | **P+ better** |
| P+ vs H3 | commits_per_s | 1008 | 69.4 | 649.0 | 74.5 | 72.0 | 77.7 | -35.6% | 4.98 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 825.3 | 137.6 | 122.2 | 3.46 | 97.4 | 1576 | -85.2% | 7.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.8 | 0.03 | 110.3 | 0.11 | 0.08 | 0.52 | +146.3% | 783.1 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -773.9 | 173.6 | 19035 | 3989 | 2823 | 40939 | -2559.6% | 7.02 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 369.0 | 35.6 | 665.9 | 27.3 | 31.8 | 119.9 | +80.5% | 9.35 | **N better** |
| N vs P+ | q1_p99_us | 1050 | 123.9 | 2162 | 544.1 | 394.6 | 498.7 | +106.0% | 2.82 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 88485 | 12081 | 503.8 | 33.1 | 8543 | 13912 | -99.4% | 10.3 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 238.0 | 58.0 | 288.5 | 41.0 | 50.2 | 43.7 | +21.2% | 1.01 | no difference |
| N vs P+ | q4_p50_us | 20431 | 592.6 | 11721 | 865.7 | 741.8 | 1325 | -42.6% | 11.7 | **P+ better** |
| N vs P+ | q5_p50_us | 14857 | 1529 | 16924 | 3490 | 2695 | 930.2 | +13.9% | 0.77 | no difference |
| N vs P+ | q6_p50_us | 54173 | 770.2 | 42286 | 2474 | 1832 | 1851 | -21.9% | 6.49 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 242.7 | 27.8 | 746.5 | 80.6 | 60.3 | 128.6 | +207.6% | 8.36 | **N better** |
| N vs P+ | c2_q1_p99_us | 1375 | 716.3 | 5327 | 422.5 | 588.1 | 2077 | +287.5% | 6.72 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 90180 | 3533 | 638.8 | 121.3 | 2500 | 2594 | -99.3% | 35.8 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 235.8 | 37.2 | 2002 | 280.6 | 200.1 | 177.0 | +748.9% | 8.82 | **N better** |
| N vs P+ | c4_q1_p99_us | 8714 | 171.8 | 9422 | 1427 | 1017 | 3913 | +8.1% | 0.70 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 152980 | 13892 | 1705 | 228.8 | 9824 | 50825 | -98.9% | 15.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 333196 | 24020 | 13070 | 7719 | 17840 | 12159 | -96.1% | 17.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 809.8 | 32.5 | 866.6 | 67.2 | 52.8 | 80.9 | +7.0% | 1.08 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1159 | 140.4 | 2867 | 441.1 | 327.4 | 2420 | +147.4% | 5.22 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1221 | 76.3 | 967.0 | 104.8 | 91.7 | 237.9 | -20.8% | 2.77 | no difference |
| N vs P+ | view_state_bytes_per_key_read | 653.8 | 121.8 | 840.6 | 150.9 | 137.1 | 1965 | +28.6% | 1.36 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 340.8 | 0.45 | 44.8 | 0.05 | 0.32 | 0.92 | -86.9% | 930.5 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 105022 | 21976 | -721.1 | 155.9 | 15540 | 220494 | -100.7% | 6.80 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 369.0 | 35.6 | 103.3 | 5.07 | 25.5 | 24.7 | -72.0% | 10.4 | **P better** |
| N vs P | q1_p99_us | 1050 | 123.9 | 634.7 | 175.9 | 152.2 | 137.9 | -39.5% | 2.73 | no difference |
| N vs P | q2_p50_us | 88485 | 12081 | 190.3 | 16.0 | 8543 | 13912 | -99.8% | 10.3 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 238.0 | 58.0 | 259.9 | 35.3 | 48.0 | 177.8 | +9.2% | 0.46 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q4_p50_us | 20431 | 592.6 | 933.0 | 75.4 | 422.4 | 297.0 | -95.4% | 46.2 | **P better** |
| N vs P | q5_p50_us | 14857 | 1529 | 153.8 | 22.0 | 1082 | 819.8 | -99.0% | 13.6 | **P better** |
| N vs P | q6_p50_us | 54173 | 770.2 | 3615 | 225.1 | 567.4 | 77.1 | -93.3% | 89.1 | **P better** |
| N vs P | c2_q1_p50_us | 242.7 | 27.8 | 78.2 | 34.2 | 31.2 | 36.2 | -67.8% | 5.27 | **P better** |
| N vs P | c2_q1_p99_us | 1375 | 716.3 | 1809 | 589.6 | 656.0 | 2126 | +31.6% | 0.66 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 90180 | 3533 | 172.7 | 33.8 | 2498 | 2595 | -99.8% | 36.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 235.8 | 37.2 | 94.4 | 16.4 | 28.7 | 23.3 | -60.0% | 4.92 | **P better** |
| N vs P | c4_q1_p99_us | 8714 | 171.8 | 2386 | 434.4 | 330.3 | 921.5 | -72.6% | 19.2 | **P better** |
| N vs P | c4_q2_p50_us | 152980 | 13892 | 233.5 | 61.4 | 9823 | 50824 | -99.8% | 15.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 333196 | 24020 | 3561 | 1038 | 17001 | 12093 | -98.9% | 19.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 809.8 | 32.5 | 150720 | 6229 | 4405 | 8967 | +18511.6% | 34.0 | **N better** |
| N vs P | write_p99_us | 1159 | 140.4 | 200016 | 23135 | 16359 | 31132 | +17157.6% | 12.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1221 | 76.3 | 6.46 | 0.35 | 54.0 | 18.1 | -99.5% | 22.5 | **N better** |
| N vs P | view_state_bytes_per_key_read | 653.8 | 121.8 | 511.8 | 105.8 | 114.1 | 1670 | -21.7% | 1.24 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 340.8 | 0.45 | 45.0 | 0.05 | 0.32 | 1.04 | -86.8% | 930.3 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 105022 | 21976 | -671.2 | 167.0 | 15540 | 220516 | -100.6% | 6.80 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 665.9 | 27.3 | 103.3 | 5.07 | 19.7 | 117.4 | -84.5% | 28.6 | **P better** |
| P+ vs P | q1_p99_us | 2162 | 544.1 | 634.7 | 175.9 | 404.3 | 490.1 | -70.6% | 3.78 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 503.8 | 33.1 | 190.3 | 16.0 | 26.0 | 93.8 | -62.2% | 12.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 288.5 | 41.0 | 259.9 | 35.3 | 38.3 | 181.0 | -9.9% | 0.75 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 11721 | 865.7 | 933.0 | 75.4 | 614.5 | 1292 | -92.0% | 17.6 | **P better** |
| P+ vs P | q5_p50_us | 16924 | 3490 | 153.8 | 22.0 | 2468 | 439.8 | -99.1% | 6.79 | **P better** |
| P+ vs P | q6_p50_us | 42286 | 2474 | 3615 | 225.1 | 1756 | 1850 | -91.5% | 22.0 | **P better** |
| P+ vs P | c2_q1_p50_us | 746.5 | 80.6 | 78.2 | 34.2 | 61.9 | 133.6 | -89.5% | 10.8 | **P better** |
| P+ vs P | c2_q1_p99_us | 5327 | 422.5 | 1809 | 589.6 | 512.9 | 522.8 | -66.0% | 6.86 | **P better** |
| P+ vs P | c2_q2_p50_us | 638.8 | 121.3 | 172.7 | 33.8 | 89.1 | 124.6 | -73.0% | 5.23 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 2002 | 280.6 | 94.4 | 16.4 | 198.7 | 177.2 | -95.3% | 9.60 | **P better** |
| P+ vs P | c4_q1_p99_us | 9422 | 1427 | 2386 | 434.4 | 1055 | 3950 | -74.7% | 6.67 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1705 | 228.8 | 233.5 | 61.4 | 167.5 | 216.6 | -86.3% | 8.79 | **P better** |
| P+ vs P | c4_q2_p99_us | 13070 | 7719 | 3561 | 1038 | 5507 | 5352 | -72.8% | 1.73 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 866.6 | 67.2 | 150720 | 6229 | 4405 | 8967 | +17291.2% | 34.0 | **P+ better** |
| P+ vs P | write_p99_us | 2867 | 441.1 | 200016 | 23135 | 16362 | 31041 | +6876.8% | 12.0 | **P+ better** |
| P+ vs P | commits_per_s | 967.0 | 104.8 | 6.46 | 0.35 | 74.1 | 237.2 | -99.3% | 13.0 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 840.6 | 150.9 | 511.8 | 105.8 | 130.3 | 1886 | -39.1% | 2.52 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.8 | 0.05 | 45.0 | 0.05 | 0.05 | 0.49 | +0.5% | 5.03 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -721.1 | 155.9 | -671.2 | 167.0 | 161.5 | 3953 | -6.9% | 0.31 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 103.3 | 5.07 | 103.6 | 4.68 | 4.88 | 4.77 | +0.2% | 0.04 | BELOW FLOOR |
| P vs M | q1_p99_us | 634.7 | 175.9 | 645.4 | 148.4 | 162.7 | 396.6 | +1.7% | 0.07 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 190.3 | 16.0 | 201.7 | 11.3 | 13.9 | 12.7 | +6.0% | 0.83 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 259.9 | 35.3 | 244.6 | 39.1 | 37.2 | 200.6 | -5.9% | 0.41 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 933.0 | 75.4 | 927.6 | 75.9 | 75.6 | 135.7 | -0.6% | 0.07 | BELOW FLOOR |
| P vs M | q5_p50_us | 153.8 | 22.0 | 150.2 | 25.4 | 23.7 | 10.5 | -2.4% | 0.15 | BELOW FLOOR |
| P vs M | q6_p50_us | 3615 | 225.1 | 3769 | 417.4 | 335.4 | 522.3 | +4.3% | 0.46 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 78.2 | 34.2 | 92.8 | 46.2 | 40.7 | 36.3 | +18.7% | 0.36 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1809 | 589.6 | 1444 | 306.5 | 469.9 | 570.6 | -20.2% | 0.78 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 172.7 | 33.8 | 231.1 | 24.1 | 29.3 | 126.4 | +33.8% | 1.99 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p50_us | 94.4 | 16.4 | 101.0 | 48.5 | 36.2 | 177.7 | +7.0% | 0.18 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2386 | 434.4 | 2554 | 482.3 | 458.9 | 995.4 | +7.0% | 0.37 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 233.5 | 61.4 | 227.5 | 60.4 | 60.9 | 266.5 | -2.6% | 0.10 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p99_us | 3561 | 1038 | 3264 | 619.9 | 854.7 | 3678 | -8.3% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 150720 | 6229 | 150617 | 2728 | 4809 | 9090 | -0.1% | 0.02 | BELOW FLOOR |
| P vs M | write_p99_us | 200016 | 23135 | 210558 | 30964 | 27331 | 31807 | +5.3% | 0.39 | BELOW FLOOR |
| P vs M | commits_per_s | 6.46 | 0.35 | 6.51 | 0.24 | 0.30 | 0.44 | +0.7% | 0.16 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 511.8 | 105.8 | 514.4 | 105.4 | 105.6 | 1563 | +0.5% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 45.0 | 0.05 | 44.8 | 0.01 | 0.03 | 0.49 | -0.5% | 6.04 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -671.2 | 167.0 | -731.5 | 177.0 | 172.0 | 3946 | +9.0% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 369.0 | 35.6 | 356.7 | 64.5 | 52.1 | 51.6 | -3.3% | 0.24 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 1050 | 123.9 | 1364 | 280.9 | 217.1 | 809.3 | +29.9% | 1.45 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 88485 | 12081 | 339.0 | 32.1 | 8543 | 13912 | -99.6% | 10.3 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 238.0 | 58.0 | 284.9 | 50.2 | 54.2 | 25.2 | +19.7% | 0.87 | no difference |
| N vs H3 | q4_p50_us | 20431 | 592.6 | 11321 | 569.8 | 581.3 | 495.8 | -44.6% | 15.7 | **H3 better** |
| N vs H3 | q5_p50_us | 14857 | 1529 | 16177 | 3273 | 2554 | 4362 | +8.9% | 0.52 | BELOW FLOOR |
| N vs H3 | q6_p50_us | 54173 | 770.2 | 45644 | 2185 | 1638 | 1011 | -15.7% | 5.21 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 242.7 | 27.8 | 345.5 | 39.7 | 34.3 | 51.6 | +42.4% | 3.00 | **N better** |
| N vs H3 | c2_q1_p99_us | 1375 | 716.3 | 1345 | 339.3 | 560.5 | 2319 | -2.2% | 0.05 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 90180 | 3533 | 372.9 | 67.0 | 2499 | 2594 | -99.6% | 35.9 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 235.8 | 37.2 | 599.9 | 58.8 | 49.2 | 317.5 | +154.4% | 7.40 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 8714 | 171.8 | 2524 | 251.9 | 215.6 | 1318 | -71.0% | 28.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 152980 | 13892 | 641.2 | 103.2 | 9823 | 50825 | -99.6% | 15.5 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p99_us | 333196 | 24020 | 2327 | 393.9 | 16987 | 11653 | -99.3% | 19.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 809.8 | 32.5 | 1303 | 77.7 | 59.6 | 80.3 | +60.9% | 8.28 | **N better** |
| N vs H3 | write_p99_us | 1159 | 140.4 | 3881 | 300.8 | 234.7 | 2508 | +234.9% | 11.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1221 | 76.3 | 663.4 | 51.6 | 65.1 | 59.4 | -45.7% | 8.56 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 653.8 | 121.8 | 122.2 | 4.54 | 86.2 | 1244 | -81.3% | 6.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 340.8 | 0.45 | 109.6 | 0.13 | 0.33 | 1.04 | -67.8% | 701.5 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 105022 | 21976 | 18880 | 3810 | 15771 | 224000 | -82.0% | 5.46 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 665.9 | 27.3 | 356.7 | 64.5 | 49.5 | 125.9 | -46.4% | 6.24 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2162 | 544.1 | 1364 | 280.9 | 433.0 | 936.0 | -36.9% | 1.84 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 503.8 | 33.1 | 339.0 | 32.1 | 32.6 | 94.2 | -32.7% | 5.06 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 288.5 | 41.0 | 284.9 | 50.2 | 45.8 | 42.4 | -1.2% | 0.08 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 11721 | 865.7 | 11321 | 569.8 | 732.8 | 1351 | -3.4% | 0.55 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 16924 | 3490 | 16177 | 3273 | 3383 | 4307 | -4.4% | 0.22 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 42286 | 2474 | 45644 | 2185 | 2334 | 2107 | +7.9% | 1.44 | no difference |
| P+ vs H3 | c2_q1_p50_us | 746.5 | 80.6 | 345.5 | 39.7 | 63.5 | 138.5 | -53.7% | 6.31 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5327 | 422.5 | 1345 | 339.3 | 383.2 | 1062 | -74.8% | 10.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 638.8 | 121.3 | 372.9 | 67.0 | 98.0 | 102.1 | -41.6% | 2.71 | no difference |
| P+ vs H3 | c4_q1_p50_us | 2002 | 280.6 | 599.9 | 58.8 | 202.7 | 362.8 | -70.0% | 6.92 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 9422 | 1427 | 2524 | 251.9 | 1025 | 4061 | -73.2% | 6.73 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1705 | 228.8 | 641.2 | 103.2 | 177.5 | 287.8 | -62.4% | 5.99 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 13070 | 7719 | 2327 | 393.9 | 5465 | 4265 | -82.2% | 1.97 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 866.6 | 67.2 | 1303 | 77.7 | 72.7 | 35.4 | +50.3% | 6.00 | **P+ better** |
| P+ vs H3 | write_p99_us | 2867 | 441.1 | 3881 | 300.8 | 377.5 | 818.8 | +35.4% | 2.69 | no difference |
| P+ vs H3 | commits_per_s | 967.0 | 104.8 | 663.4 | 51.6 | 82.6 | 243.9 | -31.4% | 3.67 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 840.6 | 150.9 | 122.2 | 4.54 | 106.7 | 1522 | -85.5% | 6.73 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.8 | 0.05 | 109.6 | 0.13 | 0.10 | 0.49 | +144.8% | 657.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -721.1 | 155.9 | 18880 | 3810 | 2696 | 39549 | -2718.1% | 7.27 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 305.8 | 12.2 | 707.5 | 54.3 | 39.3 | 82.2 | +131.4% | 10.2 | **N better** |
| N vs P+ | q1_p99_us | 939.5 | 99.9 | 1719 | 448.3 | 324.7 | 529.4 | +83.0% | 2.40 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 84834 | 13866 | 519.1 | 67.8 | 9805 | 62.6 | -99.4% | 8.60 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 224.4 | 14.2 | 267.7 | 14.0 | 14.1 | 77.9 | +19.3% | 3.06 | BELOW FLOOR |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q5_p50_us | 19293 | 6054 | 28437 | 6698 | 6384 | 2158 | +47.4% | 1.43 | no difference |
| N vs P+ | q6_p50_us | 52966 | 2387 | 39569 | 944.6 | 1815 | 9137 | -25.3% | 7.38 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 226.5 | 34.1 | 673.9 | 32.7 | 33.4 | 77.2 | +197.5% | 13.4 | **N better** |
| N vs P+ | c2_q1_p99_us | 1890 | 1110 | 5654 | 526.0 | 868.4 | 3961 | +199.1% | 4.33 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 85907 | 6396 | 568.5 | 31.7 | 4523 | 36221 | -99.3% | 18.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c4_q1_p50_us | 219.7 | 33.3 | 1972 | 81.7 | 62.4 | 232.3 | +797.5% | 28.1 | **N better** |
| N vs P+ | c4_q1_p99_us | 9069 | 320.8 | 7898 | 985.7 | 733.0 | 1642 | -12.9% | 1.60 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 152521 | 8609 | 1703 | 145.3 | 6089 | 235.8 | -98.9% | 24.8 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 357794 | 17484 | 8570 | 1391 | 12402 | 42367 | -97.6% | 28.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 737.0 | 27.6 | 859.2 | 86.8 | 64.4 | 275.5 | +16.6% | 1.90 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p99_us | 1419 | 246.8 | 2886 | 407.2 | 336.7 | 7012 | +103.3% | 4.35 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1230 | 50.5 | 934.9 | 102.3 | 80.7 | 605.7 | -24.0% | 3.66 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 651.1 | 121.3 | 822.2 | 129.9 | 125.7 | 1929 | +26.3% | 1.36 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 308.5 | 0.73 | 44.8 | 0.04 | 0.52 | 30.8 | -85.5% | 510.0 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 93824 | 19596 | -768.6 | 172.1 | 13857 | 119772 | -100.8% | 6.83 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 305.8 | 12.2 | 97.7 | 4.61 | 9.21 | 55.0 | -68.1% | 22.6 | **P better** |
| N vs P | q1_p99_us | 939.5 | 99.9 | 580.9 | 98.3 | 99.1 | 256.3 | -38.2% | 3.62 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 84834 | 13866 | 179.7 | 19.6 | 9804 | 82.2 | -99.8% | 8.63 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 224.4 | 14.2 | 208.4 | 27.8 | 22.1 | 65.8 | -7.1% | 0.72 | BELOW FLOOR |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q5_p50_us | 19293 | 6054 | 153.9 | 27.0 | 4281 | 1707 | -99.2% | 4.47 | **P better** |
| N vs P | q6_p50_us | 52966 | 2387 | 3221 | 164.9 | 1692 | 8819 | -93.9% | 29.4 | **P better** |
| N vs P | c2_q1_p50_us | 226.5 | 34.1 | 80.2 | 13.0 | 25.8 | 16.5 | -64.6% | 5.67 | **P better** |
| N vs P | c2_q1_p99_us | 1890 | 1110 | 1420 | 479.0 | 854.7 | 3977 | -24.9% | 0.55 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 85907 | 6396 | 176.1 | 54.0 | 4523 | 36221 | -99.8% | 19.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c4_q1_p50_us | 219.7 | 33.3 | 130.0 | 24.2 | 29.1 | 30.2 | -40.8% | 3.08 | **P better** |
| N vs P | c4_q1_p99_us | 9069 | 320.8 | 2729 | 381.1 | 352.2 | 1105 | -69.9% | 18.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 152521 | 8609 | 239.0 | 33.1 | 6088 | 150.4 | -99.8% | 25.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 357794 | 17484 | 3411 | 791.0 | 12376 | 41787 | -99.0% | 28.6 | **P better** |
| N vs P | write_p50_us | 737.0 | 27.6 | 143266 | 3826 | 2705 | 3791 | +19339.4% | 52.7 | **N better** |
| N vs P | write_p99_us | 1419 | 246.8 | 188788 | 16031 | 11337 | 6992 | +13201.2% | 16.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1230 | 50.5 | 6.84 | 0.24 | 35.7 | 563.0 | -99.4% | 34.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | view_state_bytes_per_key_read | 651.1 | 121.3 | 512.8 | 105.3 | 113.5 | 1666 | -21.2% | 1.22 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 308.5 | 0.73 | 44.9 | 0.02 | 0.52 | 30.8 | -85.4% | 510.3 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 93824 | 19596 | -676.0 | 158.3 | 13857 | 119770 | -100.7% | 6.82 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 707.5 | 54.3 | 97.7 | 4.61 | 38.5 | 61.1 | -86.2% | 15.8 | **P better** |
| P+ vs P | q1_p99_us | 1719 | 448.3 | 580.9 | 98.3 | 324.5 | 463.5 | -66.2% | 3.51 | **P better** |
| P+ vs P | q2_p50_us | 519.1 | 67.8 | 179.7 | 19.6 | 49.9 | 63.1 | -65.4% | 6.80 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 267.7 | 14.0 | 208.4 | 27.8 | 22.0 | 75.4 | -22.1% | 2.69 | BELOW FLOOR |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q5_p50_us | 28437 | 6698 | 153.9 | 27.0 | 4736 | 1321 | -99.5% | 5.97 | **P better** |
| P+ vs P | q6_p50_us | 39569 | 944.6 | 3221 | 164.9 | 678.1 | 2399 | -91.9% | 53.6 | **P better** |
| P+ vs P | c2_q1_p50_us | 673.9 | 32.7 | 80.2 | 13.0 | 24.9 | 75.5 | -88.1% | 23.9 | **P better** |
| P+ vs P | c2_q1_p99_us | 5654 | 526.0 | 1420 | 479.0 | 503.0 | 754.2 | -74.9% | 8.42 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 568.5 | 31.7 | 176.1 | 54.0 | 44.3 | 33.6 | -69.0% | 8.86 | **P better** |
| P+ vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c4_q1_p50_us | 1972 | 81.7 | 130.0 | 24.2 | 60.2 | 231.8 | -93.4% | 30.6 | **P better** |
| P+ vs P | c4_q1_p99_us | 7898 | 985.7 | 2729 | 381.1 | 747.3 | 1823 | -65.4% | 6.92 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1703 | 145.3 | 239.0 | 33.1 | 105.3 | 211.2 | -86.0% | 13.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 8570 | 1391 | 3411 | 791.0 | 1132 | 6998 | -60.2% | 4.56 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 859.2 | 86.8 | 143266 | 3826 | 2706 | 3801 | +16574.4% | 52.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p99_us | 2886 | 407.2 | 188788 | 16031 | 11339 | 695.6 | +6442.6% | 16.4 | **P+ better** |
| P+ vs P | commits_per_s | 934.9 | 102.3 | 6.84 | 0.24 | 72.3 | 223.3 | -99.3% | 12.8 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 822.2 | 129.9 | 512.8 | 105.3 | 118.2 | 1829 | -37.6% | 2.62 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.8 | 0.04 | 44.9 | 0.02 | 0.03 | 0.12 | +0.2% | 3.44 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -768.6 | 172.1 | -676.0 | 158.3 | 165.3 | 2434 | -12.0% | 0.56 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 97.7 | 4.61 | 97.2 | 6.07 | 5.39 | 10.5 | -0.5% | 0.09 | BELOW FLOOR |
| P vs M | q1_p99_us | 580.9 | 98.3 | 534.7 | 67.7 | 84.4 | 103.7 | -7.9% | 0.55 | BELOW FLOOR |
| P vs M | q2_p50_us | 179.7 | 19.6 | 174.7 | 15.4 | 17.6 | 58.6 | -2.8% | 0.29 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 208.4 | 27.8 | 218.1 | 22.9 | 25.5 | 48.8 | +4.7% | 0.38 | BELOW FLOOR |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q5_p50_us | 153.9 | 27.0 | 171.3 | 28.1 | 27.5 | 64.5 | +11.3% | 0.63 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | 3221 | 164.9 | 3406 | 185.8 | 175.7 | 315.5 | +5.8% | 1.06 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 80.2 | 13.0 | 32.2 | 4.58 | 9.75 | 45.6 | -59.8% | 4.92 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1420 | 479.0 | 1240 | 194.5 | 365.6 | 660.5 | -12.7% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 176.1 | 54.0 | 115.2 | 31.3 | 44.1 | 88.7 | -34.6% | 1.38 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c4_q1_p50_us | 130.0 | 24.2 | 86.2 | 15.6 | 20.4 | 47.9 | -33.7% | 2.15 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2729 | 381.1 | 2198 | 366.6 | 373.9 | 1272 | -19.5% | 1.42 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p50_us | 239.0 | 33.1 | 209.4 | 41.8 | 37.7 | 76.8 | -12.4% | 0.79 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 3411 | 791.0 | 3745 | 851.4 | 821.8 | 295.5 | +9.8% | 0.41 | no difference |
| P vs M | write_p50_us | 143266 | 3826 | 145960 | 1653 | 2947 | 8650 | +1.9% | 0.91 | BELOW FLOOR |
| P vs M | write_p99_us | 188788 | 16031 | 179218 | 12242 | 14263 | 29142 | -5.1% | 0.67 | BELOW FLOOR |
| P vs M | commits_per_s | 6.84 | 0.24 | 6.73 | 0.20 | 0.22 | 0.33 | -1.6% | 0.50 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 512.8 | 105.3 | 512.8 | 106.3 | 105.8 | 1548 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.9 | 0.02 | 44.9 | 0.07 | 0.05 | 0.09 | -0.0% | 0.19 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -676.0 | 158.3 | -690.8 | 147.0 | 152.8 | 2328 | +2.2% | 0.10 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 305.8 | 12.2 | 346.7 | 48.1 | 35.1 | 55.1 | +13.4% | 1.17 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 939.5 | 99.9 | 1291 | 176.4 | 143.3 | 454.0 | +37.5% | 2.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 84834 | 13866 | 320.6 | 49.4 | 9804 | 82.9 | -99.6% | 8.62 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 224.4 | 14.2 | 276.3 | 54.7 | 39.9 | 56.4 | +23.1% | 1.30 | BELOW FLOOR |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q5_p50_us | 19293 | 6054 | 26730 | 6202 | 6128 | 1877 | +38.5% | 1.21 | no difference |
| N vs H3 | q6_p50_us | 52966 | 2387 | 45885 | 1601 | 2032 | 9039 | -13.4% | 3.48 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 226.5 | 34.1 | 349.0 | 47.4 | 41.3 | 16.4 | +54.1% | 2.97 | no difference |
| N vs H3 | c2_q1_p99_us | 1890 | 1110 | 1466 | 273.8 | 808.2 | 3962 | -22.4% | 0.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 85907 | 6396 | 360.0 | 52.8 | 4523 | 36221 | -99.6% | 18.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c4_q1_p50_us | 219.7 | 33.3 | 619.9 | 57.0 | 46.7 | 102.7 | +182.2% | 8.57 | **N better** |
| N vs H3 | c4_q1_p99_us | 9069 | 320.8 | 2396 | 154.6 | 251.8 | 676.4 | -73.6% | 26.5 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 152521 | 8609 | 669.5 | 85.8 | 6088 | 479.9 | -99.6% | 24.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 357794 | 17484 | 2288 | 186.8 | 12364 | 41802 | -99.4% | 28.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 737.0 | 27.6 | 1237 | 120.6 | 87.5 | 199.8 | +67.9% | 5.72 | **N better** |
| N vs H3 | write_p99_us | 1419 | 246.8 | 4417 | 560.7 | 433.2 | 7018 | +211.2% | 6.92 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1230 | 50.5 | 723.8 | 75.1 | 64.0 | 568.9 | -41.2% | 7.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 651.1 | 121.3 | 122.1 | 4.21 | 85.8 | 1256 | -81.3% | 6.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 308.5 | 0.73 | 114.8 | 0.18 | 0.53 | 30.8 | -62.8% | 364.0 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 93824 | 19596 | 20659 | 4330 | 14190 | 124255 | -78.0% | 5.16 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 707.5 | 54.3 | 346.7 | 48.1 | 51.3 | 61.2 | -51.0% | 7.03 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1719 | 448.3 | 1291 | 176.4 | 340.6 | 596.1 | -24.9% | 1.26 | BELOW FLOOR |
| P+ vs H3 | q2_p50_us | 519.1 | 67.8 | 320.6 | 49.4 | 59.3 | 64.0 | -38.2% | 3.35 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 267.7 | 14.0 | 276.3 | 54.7 | 39.9 | 67.3 | +3.2% | 0.22 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q5_p50_us | 28437 | 6698 | 26730 | 6202 | 6455 | 1535 | -6.0% | 0.26 | no difference |
| P+ vs H3 | q6_p50_us | 39569 | 944.6 | 45885 | 1601 | 1314 | 3110 | +16.0% | 4.81 | **P+ better** |
| P+ vs H3 | c2_q1_p50_us | 673.9 | 32.7 | 349.0 | 47.4 | 40.7 | 75.5 | -48.2% | 7.98 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5654 | 526.0 | 1466 | 273.8 | 419.3 | 667.0 | -74.1% | 9.99 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 568.5 | 31.7 | 360.0 | 52.8 | 43.6 | 48.0 | -36.7% | 4.79 | **H3 better** |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c4_q1_p50_us | 1972 | 81.7 | 619.9 | 57.0 | 70.4 | 251.7 | -68.6% | 19.2 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 7898 | 985.7 | 2396 | 154.6 | 705.5 | 1599 | -69.7% | 7.80 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1703 | 145.3 | 669.5 | 85.8 | 119.3 | 502.3 | -60.7% | 8.67 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 8570 | 1391 | 2288 | 186.8 | 992.6 | 7087 | -73.3% | 6.33 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 859.2 | 86.8 | 1237 | 120.6 | 105.1 | 337.8 | +44.0% | 3.60 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p99_us | 2886 | 407.2 | 4417 | 560.7 | 490.0 | 920.6 | +53.1% | 3.12 | **P+ better** |
| P+ vs H3 | commits_per_s | 934.9 | 102.3 | 723.8 | 75.1 | 89.7 | 237.7 | -22.6% | 2.35 | BELOW FLOOR |
| P+ vs H3 | view_state_bytes_per_key_read | 822.2 | 129.9 | 122.1 | 4.21 | 91.9 | 1466 | -85.2% | 7.62 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.8 | 0.04 | 114.8 | 0.18 | 0.13 | 1.14 | +156.2% | 534.9 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -768.6 | 172.1 | 20659 | 4330 | 3064 | 33174 | -2787.7% | 6.99 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 325.0 | 18.2 | 636.5 | 69.3 | 50.7 | 117.9 | +95.9% | 6.15 | **N better** |
| N vs P+ | q1_p99_us | 980.0 | 228.6 | 2178 | 859.1 | 628.6 | 6029 | +122.3% | 1.91 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 80180 | 10491 | 487.8 | 61.5 | 7419 | 4203 | -99.4% | 10.7 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 231.5 | 18.4 | 287.6 | 30.3 | 25.1 | 46.2 | +24.2% | 2.23 | no difference |
| N vs P+ | q4_p50_us | 20047 | 305.8 | 10839 | 504.4 | 417.1 | 925.1 | -45.9% | 22.1 | **P+ better** |
| N vs P+ | q5_p50_us | 20028 | 6856 | 14520 | 1394 | 4947 | 5226 | -27.5% | 1.11 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q6_p50_us | 53864 | 687.0 | 41329 | 1130 | 935.0 | 1350 | -23.3% | 13.4 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 227.3 | 23.9 | 633.4 | 68.8 | 51.5 | 114.7 | +178.6% | 7.88 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 1527 | 852.4 | 5586 | 618.6 | 744.7 | 2104 | +265.8% | 5.45 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 92926 | 7338 | 544.2 | 76.8 | 5189 | 11835 | -99.4% | 17.8 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 229.1 | 24.6 | 1740 | 36.9 | 31.4 | 356.4 | +659.7% | 48.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 9023 | 617.4 | 8119 | 995.4 | 828.3 | 214.9 | -10.0% | 1.09 | no difference |
| N vs P+ | c4_q2_p50_us | 161292 | 26250 | 1539 | 45.1 | 18562 | 21535 | -99.0% | 8.61 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 337559 | 26699 | 7542 | 1442 | 18907 | 10854 | -97.8% | 17.5 | **P+ better** |
| N vs P+ | write_p50_us | 759.4 | 18.4 | 906.4 | 94.7 | 68.2 | 234.4 | +19.4% | 2.16 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1479 | 219.0 | 2649 | 183.7 | 202.1 | 3155 | +79.1% | 5.79 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1209 | 55.4 | 1016 | 72.5 | 64.5 | 179.5 | -15.9% | 2.99 | no difference |
| N vs P+ | view_state_bytes_per_key_read | 657.4 | 127.1 | 833.7 | 144.2 | 135.9 | 2066 | +26.8% | 1.30 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 308.3 | 0.53 | 44.8 | 0.02 | 0.37 | 1.70 | -85.5% | 703.5 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 94814 | 20518 | -769.2 | 170.6 | 14509 | 211434 | -100.8% | 6.59 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 325.0 | 18.2 | 102.1 | 8.74 | 14.3 | 2.87 | -68.6% | 15.6 | **P better** |
| N vs P | q1_p99_us | 980.0 | 228.6 | 534.5 | 78.3 | 170.9 | 691.7 | -45.5% | 2.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 80180 | 10491 | 194.1 | 10.0 | 7418 | 4200 | -99.8% | 10.8 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 231.5 | 18.4 | 271.0 | 55.7 | 41.5 | 20.7 | +17.1% | 0.95 | no difference |
| N vs P | q4_p50_us | 20047 | 305.8 | 834.1 | 57.7 | 220.1 | 392.2 | -95.8% | 87.3 | **P better** |
| N vs P | q5_p50_us | 20028 | 6856 | 155.1 | 28.4 | 4848 | 52.0 | -99.2% | 4.10 | **P better** |
| N vs P | q6_p50_us | 53864 | 687.0 | 3350 | 191.2 | 504.3 | 1129 | -93.8% | 100.2 | **P better** |
| N vs P | c2_q1_p50_us | 227.3 | 23.9 | 54.6 | 25.0 | 24.4 | 75.1 | -76.0% | 7.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 1527 | 852.4 | 1373 | 262.7 | 630.7 | 431.4 | -10.1% | 0.25 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 92926 | 7338 | 137.0 | 30.4 | 5189 | 11835 | -99.9% | 17.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 229.1 | 24.6 | 139.3 | 35.0 | 30.3 | 72.3 | -39.2% | 2.97 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 9023 | 617.4 | 2232 | 442.7 | 537.2 | 783.3 | -75.3% | 12.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 161292 | 26250 | 238.6 | 58.1 | 18562 | 21535 | -99.9% | 8.68 | **P better** |
| N vs P | c4_q2_p99_us | 337559 | 26699 | 2865 | 894.8 | 18890 | 10902 | -99.2% | 17.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 759.4 | 18.4 | 145975 | 2499 | 1767 | 4667 | +19121.4% | 82.2 | **N better** |
| N vs P | write_p99_us | 1479 | 219.0 | 191205 | 7667 | 5424 | 29536 | +12828.9% | 35.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1209 | 55.4 | 6.65 | 0.17 | 39.2 | 91.2 | -99.5% | 30.7 | **N better** |
| N vs P | view_state_bytes_per_key_read | 657.4 | 127.1 | 518.2 | 109.9 | 118.8 | 1765 | -21.2% | 1.17 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 308.3 | 0.53 | 45.0 | 0.02 | 0.37 | 1.70 | -85.4% | 702.9 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 94814 | 20518 | -704.7 | 158.7 | 14509 | 211434 | -100.7% | 6.58 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 636.5 | 69.3 | 102.1 | 8.74 | 49.4 | 117.9 | -84.0% | 10.8 | **P better** |
| P+ vs P | q1_p99_us | 2178 | 859.1 | 534.5 | 78.3 | 610.0 | 5991 | -75.5% | 2.70 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 487.8 | 61.5 | 194.1 | 10.0 | 44.1 | 156.7 | -60.2% | 6.66 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 287.6 | 30.3 | 271.0 | 55.7 | 44.8 | 44.3 | -5.8% | 0.37 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 10839 | 504.4 | 834.1 | 57.7 | 359.0 | 839.1 | -92.3% | 27.9 | **P better** |
| P+ vs P | q5_p50_us | 14520 | 1394 | 155.1 | 28.4 | 985.8 | 5225 | -98.9% | 14.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q6_p50_us | 41329 | 1130 | 3350 | 191.2 | 810.3 | 1634 | -91.9% | 46.9 | **P better** |
| P+ vs P | c2_q1_p50_us | 633.4 | 68.8 | 54.6 | 25.0 | 51.8 | 86.7 | -91.4% | 11.2 | **P better** |
| P+ vs P | c2_q1_p99_us | 5586 | 618.6 | 1373 | 262.7 | 475.3 | 2061 | -75.4% | 8.87 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 544.2 | 76.8 | 137.0 | 30.4 | 58.4 | 57.8 | -74.8% | 6.97 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1740 | 36.9 | 139.3 | 35.0 | 36.0 | 352.2 | -92.0% | 44.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8119 | 995.4 | 2232 | 442.7 | 770.3 | 762.9 | -72.5% | 7.64 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1539 | 45.1 | 238.6 | 58.1 | 52.0 | 136.3 | -84.5% | 25.0 | **P better** |
| P+ vs P | c4_q2_p99_us | 7542 | 1442 | 2865 | 894.8 | 1200 | 1400 | -62.0% | 3.90 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 906.4 | 94.7 | 145975 | 2499 | 1768 | 4673 | +16004.3% | 82.0 | **P+ better** |
| P+ vs P | write_p99_us | 2649 | 183.7 | 191205 | 7667 | 5423 | 29368 | +7117.3% | 34.8 | **P+ better** |
| P+ vs P | commits_per_s | 1016 | 72.5 | 6.65 | 0.17 | 51.3 | 154.6 | -99.3% | 19.7 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 833.7 | 144.2 | 518.2 | 109.9 | 128.2 | 1980 | -37.8% | 2.46 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.8 | 0.02 | 45.0 | 0.02 | 0.02 | 0.06 | +0.5% | 9.57 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -769.2 | 170.6 | -704.7 | 158.7 | 164.7 | 2475 | -8.4% | 0.39 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 102.1 | 8.74 | 101.6 | 7.56 | 8.17 | 2.54 | -0.5% | 0.06 | BELOW FLOOR |
| P vs M | q1_p99_us | 534.5 | 78.3 | 666.2 | 171.1 | 133.1 | 293.2 | +24.6% | 0.99 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 194.1 | 10.0 | 189.9 | 8.33 | 9.22 | 78.3 | -2.2% | 0.46 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 271.0 | 55.7 | 262.7 | 50.3 | 53.1 | 19.9 | -3.1% | 0.16 | BELOW FLOOR |
| P vs M | q4_p50_us | 834.1 | 57.7 | 879.3 | 68.5 | 63.3 | 35.7 | +5.4% | 0.71 | no difference |
| P vs M | q5_p50_us | 155.1 | 28.4 | 149.2 | 28.1 | 28.2 | 33.4 | -3.8% | 0.21 | BELOW FLOOR |
| P vs M | q6_p50_us | 3350 | 191.2 | 3691 | 349.2 | 281.5 | 1071 | +10.2% | 1.21 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 54.6 | 25.0 | 47.6 | 18.5 | 22.0 | 8.12 | -12.7% | 0.32 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1373 | 262.7 | 1169 | 218.6 | 241.6 | 1273 | -14.8% | 0.84 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 137.0 | 30.4 | 141.3 | 28.2 | 29.3 | 65.7 | +3.1% | 0.15 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p50_us | 139.3 | 35.0 | 93.2 | 47.8 | 41.9 | 55.1 | -33.1% | 1.10 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2232 | 442.7 | 1915 | 472.2 | 457.7 | 776.5 | -14.2% | 0.69 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 238.6 | 58.1 | 219.4 | 68.8 | 63.7 | 55.1 | -8.0% | 0.30 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2865 | 894.8 | 2743 | 575.1 | 752.1 | 1313 | -4.3% | 0.16 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 145975 | 2499 | 144127 | 1338 | 2004 | 9949 | -1.3% | 0.92 | BELOW FLOOR |
| P vs M | write_p99_us | 191205 | 7667 | 199208 | 21900 | 16408 | 29536 | +4.2% | 0.49 | BELOW FLOOR |
| P vs M | commits_per_s | 6.65 | 0.17 | 6.62 | 0.23 | 0.20 | 0.60 | -0.4% | 0.12 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 518.2 | 109.9 | 515.6 | 110.3 | 110.1 | 1664 | -0.5% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 45.0 | 0.02 | 44.9 | 0.06 | 0.04 | 0.06 | -0.1% | 0.90 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -704.7 | 158.7 | -692.8 | 161.8 | 160.3 | 2466 | -1.7% | 0.07 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 325.0 | 18.2 | 326.8 | 43.2 | 33.1 | 46.1 | +0.5% | 0.05 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 980.0 | 228.6 | 1123 | 237.3 | 233.0 | 755.5 | +14.6% | 0.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 80180 | 10491 | 353.3 | 47.9 | 7419 | 4201 | -99.6% | 10.8 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 231.5 | 18.4 | 346.7 | 45.3 | 34.6 | 90.3 | +49.7% | 3.33 | **N better** |
| N vs H3 | q4_p50_us | 20047 | 305.8 | 11178 | 898.9 | 671.4 | 395.2 | -44.2% | 13.2 | **H3 better** |
| N vs H3 | q5_p50_us | 20028 | 6856 | 17088 | 4577 | 5829 | 1889 | -14.7% | 0.50 | no difference |
| N vs H3 | q6_p50_us | 53864 | 687.0 | 45619 | 2705 | 1973 | 2428 | -15.3% | 4.18 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 227.3 | 23.9 | 382.8 | 48.8 | 38.4 | 86.8 | +68.4% | 4.05 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 1527 | 852.4 | 1466 | 605.6 | 739.3 | 1171 | -4.0% | 0.08 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 92926 | 7338 | 389.3 | 49.1 | 5189 | 11835 | -99.6% | 17.8 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 229.1 | 24.6 | 653.6 | 82.4 | 60.8 | 152.8 | +185.3% | 6.98 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 9023 | 617.4 | 2429 | 208.6 | 460.8 | 426.4 | -73.1% | 14.3 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 161292 | 26250 | 694.8 | 114.7 | 18562 | 21535 | -99.6% | 8.65 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 337559 | 26699 | 2537 | 223.9 | 18880 | 10857 | -99.2% | 17.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 759.4 | 18.4 | 1285 | 68.1 | 49.9 | 127.7 | +69.2% | 10.5 | **N better** |
| N vs H3 | write_p99_us | 1479 | 219.0 | 3999 | 507.4 | 390.8 | 3587 | +170.4% | 6.45 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1209 | 55.4 | 672.6 | 65.0 | 60.4 | 127.5 | -44.4% | 8.88 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 657.4 | 127.1 | 121.9 | 3.35 | 89.9 | 1316 | -81.5% | 5.96 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 308.3 | 0.53 | 110.1 | 0.50 | 0.52 | 1.70 | -64.3% | 383.6 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 94814 | 20518 | 19280 | 3186 | 14682 | 215848 | -79.7% | 5.14 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 636.5 | 69.3 | 326.8 | 43.2 | 57.7 | 126.6 | -48.7% | 5.37 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2178 | 859.1 | 1123 | 237.3 | 630.2 | 5999 | -48.5% | 1.68 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 487.8 | 61.5 | 353.3 | 47.9 | 55.1 | 161.6 | -27.6% | 2.44 | BELOW FLOOR |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 287.6 | 30.3 | 346.7 | 45.3 | 38.5 | 98.5 | +20.6% | 1.53 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 10839 | 504.4 | 11178 | 898.9 | 728.8 | 840.5 | +3.1% | 0.47 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 14520 | 1394 | 17088 | 4577 | 3383 | 5556 | +17.7% | 0.76 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q6_p50_us | 41329 | 1130 | 45619 | 2705 | 2073 | 2700 | +10.4% | 2.07 | no difference |
| P+ vs H3 | c2_q1_p50_us | 633.4 | 68.8 | 382.8 | 48.8 | 59.7 | 97.0 | -39.6% | 4.20 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5586 | 618.6 | 1466 | 605.6 | 612.1 | 2330 | -73.7% | 6.73 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 544.2 | 76.8 | 389.3 | 49.1 | 64.5 | 42.4 | -28.5% | 2.40 | no difference |
| P+ vs H3 | c4_q1_p50_us | 1740 | 36.9 | 653.6 | 82.4 | 63.8 | 377.0 | -62.4% | 17.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 8119 | 995.4 | 2429 | 208.6 | 719.1 | 387.7 | -70.1% | 7.91 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1539 | 45.1 | 694.8 | 114.7 | 87.1 | 127.5 | -54.8% | 9.69 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7542 | 1442 | 2537 | 223.9 | 1032 | 989.8 | -66.4% | 4.85 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 906.4 | 94.7 | 1285 | 68.1 | 82.5 | 265.2 | +41.7% | 4.59 | **P+ better** |
| P+ vs H3 | write_p99_us | 2649 | 183.7 | 3999 | 507.4 | 381.6 | 1733 | +50.9% | 3.54 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 1016 | 72.5 | 672.6 | 65.0 | 68.8 | 178.5 | -33.8% | 4.99 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 833.7 | 144.2 | 121.9 | 3.35 | 102.0 | 1594 | -85.4% | 6.98 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.8 | 0.02 | 110.1 | 0.50 | 0.36 | 0.09 | +145.9% | 183.1 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -769.2 | 170.6 | 19280 | 3186 | 2256 | 43498 | -2606.6% | 8.89 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 289.9 | 28.6 | 596.5 | 17.3 | 23.7 | 38.5 | +105.8% | 13.0 | **N better** |
| N vs P+ | q1_p99_us | 902.3 | 105.9 | 1779 | 469.4 | 340.2 | 2460 | +97.2% | 2.58 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 80537 | 3506 | 446.7 | 23.7 | 2479 | 10713 | -99.4% | 32.3 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 220.8 | 56.7 | 232.6 | 31.2 | 45.7 | 93.6 | +5.4% | 0.26 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | 20023 | 160.3 | 10454 | 249.9 | 209.9 | 4303 | -47.8% | 45.6 | **P+ better** |
| N vs P+ | q5_p50_us | 27646 | 1611 | 32800 | 2793 | 2280 | 15214 | +18.6% | 2.26 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q6_p50_us | 53231 | 1364 | 39975 | 1348 | 1356 | 6485 | -24.9% | 9.77 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 199.7 | 17.7 | 629.6 | 23.1 | 20.6 | 766.6 | +215.3% | 20.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q1_p99_us | 723.5 | 117.5 | 5726 | 1076 | 765.1 | 3571 | +691.4% | 6.54 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 86721 | 16747 | 496.8 | 43.5 | 11842 | 13237 | -99.4% | 7.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 218.1 | 12.8 | 1747 | 114.3 | 81.4 | 611.6 | +701.3% | 18.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p99_us | 8522 | 456.6 | 7566 | 886.9 | 705.3 | 4466 | -11.2% | 1.36 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 151620 | 7437 | 1511 | 101.6 | 5259 | 31591 | -99.0% | 28.5 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 328268 | 18192 | 6507 | 411.9 | 12867 | 51372 | -98.0% | 25.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 766.1 | 31.2 | 755.1 | 37.2 | 34.4 | 16.3 | -1.4% | 0.32 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1237 | 130.0 | 2694 | 233.9 | 189.2 | 4112 | +117.8% | 7.70 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1269 | 54.4 | 1082 | 39.6 | 47.6 | 473.4 | -14.7% | 3.91 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | view_state_bytes_per_key_read | 651.6 | 117.7 | 832.5 | 131.2 | 124.6 | 2006 | +27.8% | 1.45 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 351.9 | 0.43 | 44.8 | 0.01 | 0.30 | 8.23 | -87.3% | 1020 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 108587 | 22139 | -768.4 | 156.7 | 15655 | 233575 | -100.7% | 6.99 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 289.9 | 28.6 | 99.8 | 6.40 | 20.7 | 35.4 | -65.6% | 9.17 | **P better** |
| N vs P | q1_p99_us | 902.3 | 105.9 | 722.2 | 260.0 | 198.5 | 120.9 | -20.0% | 0.91 | no difference |
| N vs P | q2_p50_us | 80537 | 3506 | 194.1 | 10.3 | 2479 | 10712 | -99.8% | 32.4 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 220.8 | 56.7 | 229.9 | 15.2 | 41.5 | 133.3 | +4.2% | 0.22 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q4_p50_us | 20023 | 160.3 | 876.0 | 43.3 | 117.4 | 4302 | -95.6% | 163.1 | **P better** |
| N vs P | q5_p50_us | 27646 | 1611 | 141.9 | 20.6 | 1140 | 131.0 | -99.5% | 24.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 53231 | 1364 | 3580 | 425.4 | 1011 | 1772 | -93.3% | 49.1 | **P better** |
| N vs P | c2_q1_p50_us | 199.7 | 17.7 | 40.3 | 13.5 | 15.8 | 37.9 | -79.8% | 10.1 | **P better** |
| N vs P | c2_q1_p99_us | 723.5 | 117.5 | 1321 | 478.2 | 348.2 | 734.3 | +82.6% | 1.72 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 86721 | 16747 | 132.5 | 41.2 | 11842 | 13231 | -99.8% | 7.31 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 218.1 | 12.8 | 89.8 | 21.3 | 17.6 | 100.4 | -58.8% | 7.29 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 8522 | 456.6 | 2167 | 177.3 | 346.4 | 1014 | -74.6% | 18.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 151620 | 7437 | 209.1 | 13.0 | 5259 | 31589 | -99.9% | 28.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 328268 | 18192 | 3528 | 1179 | 12891 | 51316 | -98.9% | 25.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 766.1 | 31.2 | 145311 | 2625 | 1857 | 3538 | +18867.4% | 77.9 | **N better** |
| N vs P | write_p99_us | 1237 | 130.0 | 201911 | 6946 | 4912 | 15619 | +16228.4% | 40.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1269 | 54.4 | 6.65 | 0.29 | 38.5 | 25.1 | -99.5% | 32.8 | **N better** |
| N vs P | view_state_bytes_per_key_read | 651.6 | 117.7 | 511.1 | 103.7 | 110.9 | 1694 | -21.6% | 1.27 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 351.9 | 0.43 | 45.3 | 0.02 | 0.30 | 8.21 | -87.1% | 1016 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 108587 | 22139 | -569.7 | 117.0 | 15655 | 233503 | -100.5% | 6.97 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 596.5 | 17.3 | 99.8 | 6.40 | 13.1 | 40.3 | -83.3% | 38.0 | **P better** |
| P+ vs P | q1_p99_us | 1779 | 469.4 | 722.2 | 260.0 | 379.4 | 2458 | -59.4% | 2.79 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 446.7 | 23.7 | 194.1 | 10.3 | 18.3 | 55.7 | -56.6% | 13.8 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 232.6 | 31.2 | 229.9 | 15.2 | 24.5 | 110.3 | -1.1% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 10454 | 249.9 | 876.0 | 43.3 | 179.3 | 173.2 | -91.6% | 53.4 | **P better** |
| P+ vs P | q5_p50_us | 32800 | 2793 | 141.9 | 20.6 | 1975 | 15214 | -99.6% | 16.5 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q6_p50_us | 39975 | 1348 | 3580 | 425.4 | 999.7 | 6393 | -91.0% | 36.4 | **P better** |
| P+ vs P | c2_q1_p50_us | 629.6 | 23.1 | 40.3 | 13.5 | 18.9 | 766.5 | -93.6% | 31.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q1_p99_us | 5726 | 1076 | 1321 | 478.2 | 832.3 | 3646 | -76.9% | 5.29 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 496.8 | 43.5 | 132.5 | 41.2 | 42.4 | 410.3 | -73.3% | 8.59 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p50_us | 1747 | 114.3 | 89.8 | 21.3 | 82.2 | 617.0 | -94.9% | 20.2 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p99_us | 7566 | 886.9 | 2167 | 177.3 | 639.5 | 4543 | -71.4% | 8.44 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1511 | 101.6 | 209.1 | 13.0 | 72.5 | 348.0 | -86.2% | 18.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 6507 | 411.9 | 3528 | 1179 | 882.8 | 2662 | -45.8% | 3.37 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 755.1 | 37.2 | 145311 | 2625 | 1857 | 3538 | +19144.0% | 77.9 | **P+ better** |
| P+ vs P | write_p99_us | 2694 | 233.9 | 201911 | 6946 | 4914 | 15949 | +7396.2% | 40.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 1082 | 39.6 | 6.65 | 0.29 | 28.0 | 472.8 | -99.4% | 38.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | view_state_bytes_per_key_read | 832.5 | 131.2 | 511.1 | 103.7 | 118.3 | 1923 | -38.6% | 2.72 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.8 | 0.01 | 45.3 | 0.02 | 0.02 | 0.52 | +1.3% | 28.9 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -768.4 | 156.7 | -569.7 | 117.0 | 138.3 | 6066 | -25.9% | 1.44 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 99.8 | 6.40 | 99.6 | 8.73 | 7.65 | 26.4 | -0.2% | 0.03 | BELOW FLOOR |
| P vs M | q1_p99_us | 722.2 | 260.0 | 667.1 | 143.0 | 209.8 | 672.0 | -7.6% | 0.26 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 194.1 | 10.3 | 202.1 | 14.0 | 12.3 | 31.8 | +4.2% | 0.66 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 229.9 | 15.2 | 221.6 | 27.6 | 22.3 | 149.9 | -3.6% | 0.37 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 876.0 | 43.3 | 874.7 | 32.5 | 38.2 | 174.0 | -0.1% | 0.03 | BELOW FLOOR |
| P vs M | q5_p50_us | 141.9 | 20.6 | 138.3 | 14.7 | 17.9 | 154.1 | -2.5% | 0.20 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | 3580 | 425.4 | 3444 | 144.6 | 317.7 | 1516 | -3.8% | 0.43 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 40.3 | 13.5 | 86.0 | 17.1 | 15.4 | 26.1 | +113.5% | 2.96 | no difference |
| P vs M | c2_q1_p99_us | 1321 | 478.2 | 1276 | 210.1 | 369.3 | 1834 | -3.4% | 0.12 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 132.5 | 41.2 | 163.1 | 46.6 | 44.0 | 81.5 | +23.1% | 0.70 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p50_us | 89.8 | 21.3 | 141.9 | 49.2 | 37.9 | 91.2 | +58.0% | 1.37 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2167 | 177.3 | 2297 | 314.0 | 255.0 | 1028 | +6.0% | 0.51 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 209.1 | 13.0 | 270.7 | 66.5 | 47.9 | 74.3 | +29.5% | 1.29 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 3528 | 1179 | 2791 | 567.8 | 925.0 | 849.6 | -20.9% | 0.80 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 145311 | 2625 | 147874 | 3951 | 3354 | 3944 | +1.8% | 0.76 | BELOW FLOOR |
| P vs M | write_p99_us | 201911 | 6946 | 187104 | 15626 | 12091 | 76434 | -7.3% | 1.22 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | commits_per_s | 6.65 | 0.29 | 6.64 | 0.22 | 0.26 | 0.80 | -0.2% | 0.06 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 511.1 | 103.7 | 513.8 | 102.2 | 103.0 | 1605 | +0.5% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 45.3 | 0.02 | 44.9 | 0.04 | 0.03 | 1.01 | -1.0% | 13.9 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -569.7 | 117.0 | -719.4 | 196.5 | 161.7 | 2708 | +26.3% | 0.93 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 289.9 | 28.6 | 318.2 | 37.5 | 33.4 | 25.2 | +9.8% | 0.85 | no difference |
| N vs H3 | q1_p99_us | 902.3 | 105.9 | 998.2 | 196.6 | 157.9 | 440.7 | +10.6% | 0.61 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 80537 | 3506 | 322.0 | 51.9 | 2479 | 10713 | -99.6% | 32.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 220.8 | 56.7 | 265.9 | 32.2 | 46.1 | 91.4 | +20.4% | 0.98 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | 20023 | 160.3 | 11092 | 422.8 | 319.7 | 4362 | -44.6% | 27.9 | **H3 better** |
| N vs H3 | q5_p50_us | 27646 | 1611 | 29809 | 5401 | 3985 | 2604 | +7.8% | 0.54 | BELOW FLOOR |
| N vs H3 | q6_p50_us | 53231 | 1364 | 44821 | 2203 | 1832 | 3805 | -15.8% | 4.59 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 199.7 | 17.7 | 321.3 | 43.1 | 33.0 | 248.8 | +60.9% | 3.69 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 723.5 | 117.5 | 1205 | 259.8 | 201.6 | 1657 | +66.6% | 2.39 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 86721 | 16747 | 319.7 | 34.8 | 11842 | 13232 | -99.6% | 7.30 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 218.1 | 12.8 | 548.3 | 40.5 | 30.0 | 62.5 | +151.4% | 11.0 | **N better** |
| N vs H3 | c4_q1_p99_us | 8522 | 456.6 | 2042 | 205.2 | 354.0 | 1134 | -76.0% | 18.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 151620 | 7437 | 559.6 | 50.3 | 5259 | 31589 | -99.6% | 28.7 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 328268 | 18192 | 2113 | 442.2 | 12868 | 51320 | -99.4% | 25.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 766.1 | 31.2 | 1208 | 35.0 | 33.2 | 80.4 | +57.7% | 13.3 | **N better** |
| N vs H3 | write_p99_us | 1237 | 130.0 | 12635 | 2676 | 1894 | 2231 | +921.8% | 6.02 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1269 | 54.4 | 535.1 | 69.5 | 62.4 | 31.0 | -57.8% | 11.8 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 651.6 | 117.7 | 122.9 | 3.60 | 83.3 | 1264 | -81.1% | 6.35 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 351.9 | 0.43 | 109.6 | 0.30 | 0.37 | 8.21 | -68.8% | 659.5 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 108587 | 22139 | 19027 | 3764 | 15879 | 237312 | -82.5% | 5.64 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 596.5 | 17.3 | 318.2 | 37.5 | 29.2 | 31.6 | -46.7% | 9.53 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1779 | 469.4 | 998.2 | 196.6 | 359.8 | 2494 | -43.9% | 2.17 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 446.7 | 23.7 | 322.0 | 51.9 | 40.4 | 95.1 | -27.9% | 3.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 232.6 | 31.2 | 265.9 | 32.2 | 31.7 | 52.4 | +14.3% | 1.05 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 10454 | 249.9 | 11092 | 422.8 | 347.3 | 744.0 | +6.1% | 1.84 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 32800 | 2793 | 29809 | 5401 | 4299 | 15435 | -9.1% | 0.70 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q6_p50_us | 39975 | 1348 | 44821 | 2203 | 1826 | 7225 | +12.1% | 2.65 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 629.6 | 23.1 | 321.3 | 43.1 | 34.6 | 805.0 | -49.0% | 8.91 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q1_p99_us | 5726 | 1076 | 1205 | 259.8 | 782.4 | 3937 | -78.9% | 5.78 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 496.8 | 43.5 | 319.7 | 34.8 | 39.4 | 449.4 | -35.6% | 4.49 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q1_p50_us | 1747 | 114.3 | 548.3 | 40.5 | 85.8 | 611.9 | -68.6% | 14.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p99_us | 7566 | 886.9 | 2042 | 205.2 | 643.7 | 4571 | -73.0% | 8.58 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1511 | 101.6 | 559.6 | 50.3 | 80.2 | 343.8 | -63.0% | 11.9 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6507 | 411.9 | 2113 | 442.2 | 427.3 | 2726 | -67.5% | 10.3 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 755.1 | 37.2 | 1208 | 35.0 | 36.1 | 81.8 | +60.0% | 12.5 | **P+ better** |
| P+ vs H3 | write_p99_us | 2694 | 233.9 | 12635 | 2676 | 1899 | 3924 | +369.1% | 5.23 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | commits_per_s | 1082 | 39.6 | 535.1 | 69.5 | 56.5 | 473.1 | -50.6% | 9.68 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | view_state_bytes_per_key_read | 832.5 | 131.2 | 122.9 | 3.60 | 92.8 | 1558 | -85.2% | 7.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.8 | 0.01 | 109.6 | 0.30 | 0.21 | 0.52 | +145.0% | 308.4 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -768.4 | 156.7 | 19027 | 3764 | 2664 | 42778 | -2576.3% | 7.43 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |

## `multi`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 244.2 | 29.3 | 669.0 | 40.7 | 35.5 | 91.1 | +173.9% | 12.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 2556 | 1486 | 3932 | 739.2 | 1174 | 2200 | +53.8% | 1.17 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 908126 | 66215 | 581.5 | 53.4 | 46821 | 35998 | -99.9% | 19.4 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 346.3 | 73.4 | 285.9 | 39.4 | 58.9 | 74.1 | -17.4% | 1.03 | BELOW FLOOR |
| N vs P+ | q4_p50_us | 200989 | 5354 | 76575 | 4672 | 5024 | 5311 | -61.9% | 24.8 | **P+ better** |
| N vs P+ | q5_p50_us | 357251 | 199242 | 447513 | 343059 | 280523 | 48962 | +25.3% | 0.32 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q6_p50_us | 1223803 | 64134 | 1507667 | 64028 | 64081 | 98353 | +23.2% | 4.43 | **N better** |
| N vs P+ | c2_q1_p50_us | 114.7 | 22.9 | 711.6 | 92.8 | 67.6 | 283.9 | +520.5% | 8.83 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q1_p99_us | 644.0 | 196.4 | 4891 | 764.0 | 557.8 | 4570 | +659.4% | 7.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 860081 | 104212 | 654.4 | 109.5 | 73689 | 310100 | -99.9% | 11.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 113.2 | 17.5 | 1843 | 262.1 | 185.7 | 338.5 | +1528.0% | 9.31 | **N better** |
| N vs P+ | c4_q1_p99_us | 5749 | 138.0 | 6598 | 1163 | 827.9 | 1042 | +14.8% | 1.03 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 1359883 | 117264 | 1746 | 294.6 | 82918 | 543139 | -99.9% | 16.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 3148158 | 85343 | 7648 | 1038 | 60351 | 41856 | -99.8% | 52.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 1050 | 148.4 | 1194 | 35.2 | 107.9 | 586.7 | +13.7% | 1.33 | BELOW FLOOR |
| N vs P+ | write_p99_us | 2921 | 695.8 | 3401 | 254.2 | 523.8 | 2994 | +16.4% | 0.92 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 829.6 | 95.3 | 754.7 | 37.5 | 72.4 | 65.4 | -9.0% | 1.03 | no difference |
| N vs P+ | view_state_bytes_per_key_read | 6623 | 1632 | 4512 | 1115 | 1398 | 21656 | -31.9% | 1.51 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 3032 | 0.54 | 495.0 | 0.18 | 0.40 | 52.8 | -83.7% | 6286 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 709433 | 178046 | 46247 | 11575 | 126163 | 1690081 | -93.5% | 5.26 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 244.2 | 29.3 | 120.0 | 5.48 | 21.1 | 115.6 | -50.9% | 5.88 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q1_p99_us | 2556 | 1486 | 1627 | 621.7 | 1139 | 297.7 | -36.4% | 0.82 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 908126 | 66215 | 220.3 | 18.2 | 46821 | 35998 | -100.0% | 19.4 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 346.3 | 73.4 | 194.7 | 21.6 | 54.1 | 49.0 | -43.8% | 2.80 | no difference |
| N vs P | q4_p50_us | 200989 | 5354 | 6387 | 192.6 | 3788 | 3913 | -96.8% | 51.4 | **P better** |
| N vs P | q5_p50_us | 357251 | 199242 | 170.5 | 27.6 | 140885 | 11176 | -100.0% | 2.53 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 1223803 | 64134 | 36214 | 930.6 | 45355 | 17187 | -97.0% | 26.2 | **P better** |
| N vs P | c2_q1_p50_us | 114.7 | 22.9 | 45.0 | 8.65 | 17.3 | 31.0 | -60.7% | 4.03 | **P better** |
| N vs P | c2_q1_p99_us | 644.0 | 196.4 | 1326 | 473.1 | 362.2 | 4764 | +106.0% | 1.88 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 860081 | 104212 | 175.9 | 31.5 | 73689 | 310100 | -100.0% | 11.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 113.2 | 17.5 | 100.6 | 16.7 | 17.1 | 36.4 | -11.1% | 0.74 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 5749 | 138.0 | 3117 | 406.0 | 303.2 | 644.0 | -45.8% | 8.68 | **P better** |
| N vs P | c4_q2_p50_us | 1359883 | 117264 | 256.0 | 20.7 | 82918 | 543139 | -100.0% | 16.4 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 3148158 | 85343 | 5660 | 3002 | 60384 | 41267 | -99.8% | 52.0 | **P better** |
| N vs P | write_p50_us | 1050 | 148.4 | 2039269 | 57919 | 40955 | 6018 | +194089.2% | 49.8 | **N better** |
| N vs P | write_p99_us | 2921 | 695.8 | 2315456 | 108435 | 76677 | 31934 | +79157.5% | 30.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 829.6 | 95.3 | 0.48 | 0.01 | 67.4 | 65.4 | -99.9% | 12.3 | **N better** |
| N vs P | view_state_bytes_per_key_read | 6623 | 1632 | 3711 | 931.1 | 1329 | 20632 | -44.0% | 2.19 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 3032 | 0.54 | 490.9 | 0.61 | 0.58 | 52.9 | -83.8% | 4415 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 709433 | 178046 | 53206 | 13196 | 126243 | 1692865 | -92.5% | 5.20 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 669.0 | 40.7 | 120.0 | 5.48 | 29.0 | 96.4 | -82.1% | 18.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p99_us | 3932 | 739.2 | 1627 | 621.7 | 683.0 | 2185 | -58.6% | 3.38 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 581.5 | 53.4 | 220.3 | 18.2 | 39.9 | 124.4 | -62.1% | 9.04 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 285.9 | 39.4 | 194.7 | 21.6 | 31.8 | 70.7 | -31.9% | 2.87 | no difference |
| P+ vs P | q4_p50_us | 76575 | 4672 | 6387 | 192.6 | 3307 | 3626 | -91.7% | 21.2 | **P better** |
| P+ vs P | q5_p50_us | 447513 | 343059 | 170.5 | 27.6 | 242579 | 47670 | -100.0% | 1.84 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q6_p50_us | 1507667 | 64028 | 36214 | 930.6 | 45279 | 96848 | -97.6% | 32.5 | **P better** |
| P+ vs P | c2_q1_p50_us | 711.6 | 92.8 | 45.0 | 8.65 | 65.9 | 284.4 | -93.7% | 10.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q1_p99_us | 4891 | 764.0 | 1326 | 473.1 | 635.4 | 1639 | -72.9% | 5.61 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 654.4 | 109.5 | 175.9 | 31.5 | 80.6 | 168.0 | -73.1% | 5.94 | **P better** |
| P+ vs P | c4_q1_p50_us | 1843 | 262.1 | 100.6 | 16.7 | 185.7 | 339.7 | -94.5% | 9.38 | **P better** |
| P+ vs P | c4_q1_p99_us | 6598 | 1163 | 3117 | 406.0 | 870.8 | 1188 | -52.8% | 4.00 | **P better** |
| P+ vs P | c4_q2_p50_us | 1746 | 294.6 | 256.0 | 20.7 | 208.8 | 452.0 | -85.3% | 7.13 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 7648 | 1038 | 5660 | 3002 | 2246 | 6997 | -26.0% | 0.89 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 1194 | 35.2 | 2039269 | 57919 | 40955 | 5989 | +170690.3% | 49.8 | **P+ better** |
| P+ vs P | write_p99_us | 3401 | 254.2 | 2315456 | 108435 | 76676 | 31795 | +67980.3% | 30.2 | **P+ better** |
| P+ vs P | commits_per_s | 754.7 | 37.5 | 0.48 | 0.01 | 26.5 | 1.88 | -99.9% | 28.4 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 4512 | 1115 | 3711 | 931.1 | 1027 | 15891 | -17.8% | 0.78 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 495.0 | 0.18 | 490.9 | 0.61 | 0.45 | 3.33 | -0.8% | 9.15 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 46247 | 11575 | 53206 | 13196 | 12412 | 161514 | +15.0% | 0.56 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 120.0 | 5.48 | 117.8 | 4.40 | 4.97 | 109.4 | -1.8% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q1_p99_us | 1627 | 621.7 | 2178 | 982.3 | 822.1 | 132.1 | +33.9% | 0.67 | no difference |
| P vs M | q2_p50_us | 220.3 | 18.2 | 211.0 | 9.46 | 14.5 | 7.78 | -4.2% | 0.64 | no difference |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 194.7 | 21.6 | 244.4 | 38.6 | 31.3 | 36.7 | +25.5% | 1.59 | no difference |
| P vs M | q4_p50_us | 6387 | 192.6 | 6543 | 266.3 | 232.4 | 526.6 | +2.4% | 0.67 | BELOW FLOOR |
| P vs M | q5_p50_us | 170.5 | 27.6 | 156.2 | 17.7 | 23.1 | 205.2 | -8.4% | 0.62 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 36214 | 930.6 | 38253 | 1232 | 1092 | 929.7 | +5.6% | 1.87 | no difference |
| P vs M | c2_q1_p50_us | 45.0 | 8.65 | 62.2 | 26.2 | 19.5 | 25.2 | +38.2% | 0.88 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1326 | 473.1 | 1857 | 750.5 | 627.3 | 1499 | +40.0% | 0.85 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 175.9 | 31.5 | 165.1 | 26.9 | 29.3 | 28.1 | -6.1% | 0.37 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 100.6 | 16.7 | 101.5 | 14.9 | 15.8 | 41.6 | +0.8% | 0.05 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 3117 | 406.0 | 2861 | 673.9 | 556.3 | 905.1 | -8.2% | 0.46 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 256.0 | 20.7 | 235.3 | 29.4 | 25.5 | 139.8 | -8.1% | 0.81 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p99_us | 5660 | 3002 | 8326 | 4737 | 3966 | 556.4 | +47.1% | 0.67 | no difference |
| P vs M | write_p50_us | 2039269 | 57919 | 2066330 | 32715 | 47037 | 17450 | +1.3% | 0.58 | no difference |
| P vs M | write_p99_us | 2315456 | 108435 | 2289915 | 25127 | 78707 | 32892 | -1.1% | 0.32 | BELOW FLOOR |
| P vs M | commits_per_s | 0.48 | 0.01 | 0.48 | 0.01 | 0.01 | 0.01 | -0.5% | 0.23 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 3711 | 931.1 | 3711 | 931.1 | 931.1 | 14464 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 490.9 | 0.61 | 491.0 | 0.82 | 0.72 | 3.27 | +0.0% | 0.07 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 53206 | 13196 | 54296 | 13533 | 13365 | 194047 | +2.0% | 0.08 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 244.2 | 29.3 | 304.5 | 17.1 | 24.0 | 108.0 | +24.7% | 2.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 2556 | 1486 | 3255 | 1790 | 1645 | 1064 | +27.4% | 0.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 908126 | 66215 | 298.7 | 29.6 | 46821 | 35998 | -100.0% | 19.4 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 346.3 | 73.4 | 328.0 | 31.5 | 56.4 | 174.6 | -5.3% | 0.33 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q4_p50_us | 200989 | 5354 | 78105 | 3119 | 4381 | 14064 | -61.1% | 28.0 | **H3 better** |
| N vs H3 | q5_p50_us | 357251 | 199242 | 480543 | 373606 | 299398 | 14386 | +34.5% | 0.41 | no difference |
| N vs H3 | q6_p50_us | 1223803 | 64134 | 1590178 | 48683 | 56935 | 82756 | +29.9% | 6.43 | **N better** |
| N vs H3 | c2_q1_p50_us | 114.7 | 22.9 | 245.3 | 39.4 | 32.2 | 46.8 | +113.9% | 4.05 | **N better** |
| N vs H3 | c2_q1_p99_us | 644.0 | 196.4 | 2023 | 643.2 | 475.5 | 7008 | +214.2% | 2.90 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 860081 | 104212 | 219.6 | 36.6 | 73689 | 310100 | -100.0% | 11.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 113.2 | 17.5 | 311.4 | 66.4 | 48.5 | 56.1 | +175.0% | 4.08 | **N better** |
| N vs H3 | c4_q1_p99_us | 5749 | 138.0 | 5530 | 2514 | 1780 | 10296 | -3.8% | 0.12 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 1359883 | 117264 | 294.1 | 57.5 | 82918 | 543139 | -100.0% | 16.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 3148158 | 85343 | 4071 | 1151 | 60352 | 41267 | -99.9% | 52.1 | **H3 better** |
| N vs H3 | write_p50_us | 1050 | 148.4 | 1642 | 103.9 | 128.1 | 612.4 | +56.4% | 4.62 | BELOW FLOOR |
| N vs H3 | write_p99_us | 2921 | 695.8 | 5533 | 1543 | 1197 | 4107 | +89.4% | 2.18 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 829.6 | 95.3 | 520.6 | 39.3 | 72.9 | 93.7 | -37.2% | 4.24 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 6623 | 1632 | 193.5 | 0.61 | 1154 | 17918 | -97.1% | 5.57 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 3032 | 0.54 | 665.3 | 51.8 | 36.6 | 52.8 | -78.1% | 64.6 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 709433 | 178046 | 97688 | 13923 | 126282 | 1699143 | -86.2% | 4.84 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 669.0 | 40.7 | 304.5 | 17.1 | 31.2 | 87.1 | -54.5% | 11.7 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3932 | 739.2 | 3255 | 1790 | 1370 | 2412 | -17.2% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 581.5 | 53.4 | 298.7 | 29.6 | 43.2 | 132.0 | -48.6% | 6.55 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 285.9 | 39.4 | 328.0 | 31.5 | 35.7 | 181.9 | +14.7% | 1.18 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 76575 | 4672 | 78105 | 3119 | 3972 | 13987 | +2.0% | 0.39 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 447513 | 343059 | 480543 | 373606 | 358658 | 48523 | +7.4% | 0.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q6_p50_us | 1507667 | 64028 | 1590178 | 48683 | 56875 | 126225 | +5.5% | 1.45 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 711.6 | 92.8 | 245.3 | 39.4 | 71.3 | 286.6 | -65.5% | 6.54 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q1_p99_us | 4891 | 764.0 | 2023 | 643.2 | 706.2 | 5395 | -58.6% | 4.06 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 654.4 | 109.5 | 219.6 | 36.6 | 81.6 | 172.1 | -66.4% | 5.33 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1843 | 262.1 | 311.4 | 66.4 | 191.2 | 342.3 | -83.1% | 8.01 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6598 | 1163 | 5530 | 2514 | 1959 | 10344 | -16.2% | 0.55 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1746 | 294.6 | 294.1 | 57.5 | 212.2 | 440.9 | -83.2% | 6.84 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7648 | 1038 | 4071 | 1151 | 1096 | 7000 | -46.8% | 3.27 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1194 | 35.2 | 1642 | 103.9 | 77.6 | 181.3 | +37.5% | 5.78 | **P+ better** |
| P+ vs H3 | write_p99_us | 3401 | 254.2 | 5533 | 1543 | 1106 | 2820 | +62.7% | 1.93 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 754.7 | 37.5 | 520.6 | 39.3 | 38.4 | 67.1 | -31.0% | 6.09 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 4512 | 1115 | 193.5 | 0.61 | 788.3 | 12162 | -95.7% | 5.48 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 495.0 | 0.18 | 665.3 | 51.8 | 36.6 | 0.69 | +34.4% | 4.65 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 46247 | 11575 | 97688 | 13923 | 12803 | 217679 | +111.2% | 4.02 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 223.9 | 29.2 | 837.2 | 41.9 | 36.1 | 13.8 | +273.9% | 17.0 | **N better** |
| N vs P+ | q1_p99_us | 1538 | 612.8 | 3782 | 639.9 | 626.5 | 1051 | +146.0% | 3.58 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 804503 | 95015 | 744.9 | 37.4 | 67186 | 45049 | -99.9% | 12.0 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 256.3 | 43.3 | 332.0 | 33.4 | 38.7 | 147.2 | +29.6% | 1.96 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q5_p50_us | 181668 | 22970 | 126740 | 20301 | 21677 | 63902 | -30.2% | 2.53 | BELOW FLOOR |
| N vs P+ | q6_p50_us | 1225959 | 27230 | 1585677 | 64742 | 49664 | 69822 | +29.3% | 7.24 | **N better** |
| N vs P+ | c2_q1_p50_us | 104.3 | 8.68 | 819.3 | 70.8 | 50.4 | 362.0 | +685.4% | 14.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q1_p99_us | 639.9 | 157.3 | 5665 | 982.1 | 703.3 | 129.3 | +785.4% | 7.15 | **N better** |
| N vs P+ | c2_q2_p50_us | 811835 | 65970 | 781.0 | 51.8 | 46648 | 3366 | -99.9% | 17.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 107.3 | 13.3 | 2180 | 167.7 | 118.9 | 525.5 | +1932.6% | 17.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 6051 | 361.0 | 10848 | 2380 | 1702 | 1107 | +79.3% | 2.82 | no difference |
| N vs P+ | c4_q2_p50_us | 1338100 | 83455 | 2071 | 115.8 | 59012 | 195512 | -99.8% | 22.6 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 3021554 | 108150 | 8640 | 3718 | 76519 | 249481 | -99.7% | 39.4 | **P+ better** |
| N vs P+ | write_p50_us | 961.9 | 65.0 | 1330 | 69.2 | 67.1 | 824.8 | +38.3% | 5.48 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 2466 | 420.1 | 3716 | 587.8 | 510.9 | 1877 | +50.7% | 2.45 | BELOW FLOOR |
| N vs P+ | commits_per_s | 871.7 | 40.6 | 617.6 | 12.2 | 29.9 | 91.5 | -29.2% | 8.49 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 6663 | 1660 | 4761 | 1190 | 1444 | 22322 | -28.5% | 1.32 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 3319 | 0.71 | 495.0 | 0.17 | 0.52 | 108.6 | -85.1% | 5474 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 787732 | 200082 | 53553 | 13590 | 141805 | 1990833 | -93.2% | 5.18 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 223.9 | 29.2 | 135.3 | 12.2 | 22.4 | 91.7 | -39.6% | 3.96 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q1_p99_us | 1538 | 612.8 | 1220 | 319.6 | 488.7 | 665.3 | -20.6% | 0.65 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 804503 | 95015 | 261.9 | 16.0 | 67186 | 45049 | -100.0% | 12.0 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 256.3 | 43.3 | 310.7 | 65.2 | 55.3 | 146.2 | +21.2% | 0.98 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q5_p50_us | 181668 | 22970 | 176.6 | 25.3 | 16242 | 50582 | -99.9% | 11.2 | **P better** |
| N vs P | q6_p50_us | 1225959 | 27230 | 35989 | 910.5 | 19265 | 14337 | -97.1% | 61.8 | **P better** |
| N vs P | c2_q1_p50_us | 104.3 | 8.68 | 46.2 | 5.76 | 7.37 | 101.6 | -55.8% | 7.90 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 639.9 | 157.3 | 1687 | 490.3 | 364.1 | 26.6 | +163.7% | 2.88 | no difference |
| N vs P | c2_q2_p50_us | 811835 | 65970 | 158.8 | 24.5 | 46648 | 3359 | -100.0% | 17.4 | **P better** |
| N vs P | c4_q1_p50_us | 107.3 | 13.3 | 123.7 | 37.3 | 28.0 | 52.7 | +15.3% | 0.59 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 6051 | 361.0 | 4031 | 664.4 | 534.6 | 179.3 | -33.4% | 3.78 | **P better** |
| N vs P | c4_q2_p50_us | 1338100 | 83455 | 279.5 | 41.1 | 59012 | 195512 | -100.0% | 22.7 | **P better** |
| N vs P | c4_q2_p99_us | 3021554 | 108150 | 11000 | 4355 | 76535 | 250671 | -99.6% | 39.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 961.9 | 65.0 | 2283257 | 108212 | 76517 | 19976 | +237268.2% | 29.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p99_us | 2466 | 420.1 | 2585957 | 174698 | 123531 | 47099 | +104777.6% | 20.9 | **N better** |
| N vs P | commits_per_s | 871.7 | 40.6 | 0.44 | 0.02 | 28.7 | 89.7 | -100.0% | 30.4 | **N better** |
| N vs P | view_state_bytes_per_key_read | 6663 | 1660 | 3726 | 945.5 | 1351 | 20929 | -44.1% | 2.17 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 3319 | 0.71 | 489.8 | 0.65 | 0.68 | 108.7 | -85.2% | 4148 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 787732 | 200082 | 54360 | 13978 | 141824 | 1991662 | -93.1% | 5.17 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 837.2 | 41.9 | 135.3 | 12.2 | 30.8 | 92.5 | -83.8% | 22.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p99_us | 3782 | 639.9 | 1220 | 319.6 | 505.8 | 1205 | -67.7% | 5.07 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 744.9 | 37.4 | 261.9 | 16.0 | 28.8 | 44.9 | -64.8% | 16.8 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 332.0 | 33.4 | 310.7 | 65.2 | 51.8 | 25.6 | -6.4% | 0.41 | BELOW FLOOR |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q5_p50_us | 126740 | 20301 | 176.6 | 25.3 | 14355 | 39050 | -99.9% | 8.82 | **P better** |
| P+ vs P | q6_p50_us | 1585677 | 64742 | 35989 | 910.5 | 45784 | 68353 | -97.7% | 33.8 | **P better** |
| P+ vs P | c2_q1_p50_us | 819.3 | 70.8 | 46.2 | 5.76 | 50.2 | 373.8 | -94.4% | 15.4 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q1_p99_us | 5665 | 982.1 | 1687 | 490.3 | 776.2 | 129.6 | -70.2% | 5.12 | **P better** |
| P+ vs P | c2_q2_p50_us | 781.0 | 51.8 | 158.8 | 24.5 | 40.5 | 220.2 | -79.7% | 15.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p50_us | 2180 | 167.7 | 123.7 | 37.3 | 121.5 | 528.1 | -94.3% | 16.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 10848 | 2380 | 4031 | 664.4 | 1747 | 1109 | -62.8% | 3.90 | **P better** |
| P+ vs P | c4_q2_p50_us | 2071 | 115.8 | 279.5 | 41.1 | 86.9 | 57.5 | -86.5% | 20.6 | **P better** |
| P+ vs P | c4_q2_p99_us | 8640 | 3718 | 11000 | 4355 | 4049 | 24420 | +27.3% | 0.58 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 1330 | 69.2 | 2283257 | 108212 | 76517 | 19960 | +171585.6% | 29.8 | **P+ better** |
| P+ vs P | write_p99_us | 3716 | 587.8 | 2585957 | 174698 | 123531 | 47063 | +69486.9% | 20.9 | **P+ better** |
| P+ vs P | commits_per_s | 617.6 | 12.2 | 0.44 | 0.02 | 8.62 | 18.0 | -99.9% | 71.6 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 4761 | 1190 | 3726 | 945.5 | 1075 | 16583 | -21.7% | 0.96 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 495.0 | 0.17 | 489.8 | 0.65 | 0.48 | 3.05 | -1.1% | 10.9 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 53553 | 13590 | 54360 | 13978 | 13785 | 185907 | +1.5% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 135.3 | 12.2 | 125.1 | 7.72 | 10.2 | 103.4 | -7.5% | 0.99 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q1_p99_us | 1220 | 319.6 | 1704 | 476.9 | 405.9 | 635.5 | +39.6% | 1.19 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 261.9 | 16.0 | 256.0 | 23.4 | 20.1 | 47.3 | -2.3% | 0.30 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 310.7 | 65.2 | 275.3 | 56.8 | 61.1 | 19.2 | -11.4% | 0.58 | no difference |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q5_p50_us | 176.6 | 25.3 | 193.9 | 31.0 | 28.3 | 49.7 | +9.8% | 0.61 | BELOW FLOOR |
| P vs M | q6_p50_us | 35989 | 910.5 | 36570 | 1056 | 985.8 | 2683 | +1.6% | 0.59 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 46.2 | 5.76 | 54.5 | 20.9 | 15.3 | 103.0 | +18.0% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 1687 | 490.3 | 1459 | 549.4 | 520.7 | 156.8 | -13.5% | 0.44 | no difference |
| P vs M | c2_q2_p50_us | 158.8 | 24.5 | 208.6 | 42.1 | 34.4 | 37.9 | +31.4% | 1.45 | no difference |
| P vs M | c4_q1_p50_us | 123.7 | 37.3 | 85.8 | 8.89 | 27.1 | 52.6 | -30.7% | 1.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 4031 | 664.4 | 4086 | 726.3 | 696.0 | 1672 | +1.4% | 0.08 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 279.5 | 41.1 | 286.1 | 75.5 | 60.8 | 46.9 | +2.4% | 0.11 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 11000 | 4355 | 12709 | 8717 | 6891 | 24510 | +15.5% | 0.25 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 2283257 | 108212 | 2240368 | 101812 | 105060 | 142270 | -1.9% | 0.41 | BELOW FLOOR |
| P vs M | write_p99_us | 2585957 | 174698 | 2622577 | 220355 | 198841 | 104146 | +1.4% | 0.18 | BELOW FLOOR |
| P vs M | commits_per_s | 0.44 | 0.02 | 0.44 | 0.02 | 0.02 | 0.02 | +1.5% | 0.29 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 3726 | 945.5 | 3726 | 945.5 | 945.5 | 14655 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 489.8 | 0.65 | 489.8 | 0.88 | 0.78 | 2.71 | +0.0% | 0.04 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 54360 | 13978 | 54188 | 13595 | 13788 | 199712 | -0.3% | 0.01 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 223.9 | 29.2 | 319.3 | 22.9 | 26.2 | 114.3 | +42.6% | 3.64 | BELOW FLOOR |
| N vs H3 | q1_p99_us | 1538 | 612.8 | 2511 | 988.9 | 822.6 | 963.9 | +63.3% | 1.18 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 804503 | 95015 | 292.6 | 23.2 | 67186 | 45049 | -100.0% | 12.0 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 256.3 | 43.3 | 353.7 | 27.2 | 36.2 | 145.8 | +38.0% | 2.69 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q5_p50_us | 181668 | 22970 | 128535 | 28096 | 25661 | 61697 | -29.2% | 2.07 | BELOW FLOOR |
| N vs H3 | q6_p50_us | 1225959 | 27230 | 1700919 | 120992 | 87694 | 134357 | +38.7% | 5.42 | **N better** |
| N vs H3 | c2_q1_p50_us | 104.3 | 8.68 | 222.5 | 33.1 | 24.2 | 32.3 | +113.3% | 4.88 | **N better** |
| N vs H3 | c2_q1_p99_us | 639.9 | 157.3 | 2924 | 1583 | 1125 | 921.8 | +356.9% | 2.03 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 811835 | 65970 | 218.2 | 25.6 | 46648 | 3359 | -100.0% | 17.4 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 107.3 | 13.3 | 293.0 | 38.3 | 28.7 | 27.4 | +173.1% | 6.48 | **N better** |
| N vs H3 | c4_q1_p99_us | 6051 | 361.0 | 4146 | 2443 | 1746 | 4275 | -31.5% | 1.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 1338100 | 83455 | 301.1 | 38.9 | 59012 | 195512 | -100.0% | 22.7 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 3021554 | 108150 | 3689 | 1954 | 76486 | 249544 | -99.9% | 39.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 961.9 | 65.0 | 1642 | 73.4 | 69.3 | 817.3 | +70.7% | 9.81 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p99_us | 2466 | 420.1 | 10380 | 2222 | 1599 | 2104 | +321.0% | 4.95 | **N better** |
| N vs H3 | commits_per_s | 871.7 | 40.6 | 478.0 | 51.2 | 46.2 | 89.9 | -45.2% | 8.53 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 6663 | 1660 | 193.2 | 0.32 | 1174 | 18184 | -97.1% | 5.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 3319 | 0.71 | 690.6 | 1.01 | 0.87 | 108.8 | -79.2% | 3018 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 787732 | 200082 | 104682 | 23212 | 142428 | 1990164 | -86.7% | 4.80 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 837.2 | 41.9 | 319.3 | 22.9 | 33.7 | 115.0 | -61.9% | 15.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3782 | 639.9 | 2511 | 988.9 | 832.8 | 1392 | -33.6% | 1.53 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 744.9 | 37.4 | 292.6 | 23.2 | 31.2 | 75.2 | -60.7% | 14.5 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 332.0 | 33.4 | 353.7 | 27.2 | 30.5 | 22.9 | +6.5% | 0.71 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q5_p50_us | 126740 | 20301 | 128535 | 28096 | 24510 | 52659 | +1.4% | 0.07 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 1585677 | 64742 | 1700919 | 120992 | 97033 | 150061 | +7.3% | 1.19 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 819.3 | 70.8 | 222.5 | 33.1 | 55.3 | 361.2 | -72.8% | 10.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q1_p99_us | 5665 | 982.1 | 2924 | 1583 | 1317 | 930.5 | -48.4% | 2.08 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 781.0 | 51.8 | 218.2 | 25.6 | 40.9 | 218.9 | -72.1% | 13.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 2180 | 167.7 | 293.0 | 38.3 | 121.6 | 526.1 | -86.6% | 15.5 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 10848 | 2380 | 4146 | 2443 | 2411 | 4413 | -61.8% | 2.78 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 2071 | 115.8 | 301.1 | 38.9 | 86.4 | 46.4 | -85.5% | 20.5 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8640 | 3718 | 3689 | 1954 | 2970 | 5702 | -57.3% | 1.67 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1330 | 69.2 | 1642 | 73.4 | 71.4 | 115.1 | +23.5% | 4.37 | **P+ better** |
| P+ vs H3 | write_p99_us | 3716 | 587.8 | 10380 | 2222 | 1625 | 981.5 | +179.3% | 4.10 | **P+ better** |
| P+ vs H3 | commits_per_s | 617.6 | 12.2 | 478.0 | 51.2 | 37.2 | 18.9 | -22.6% | 3.75 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 4761 | 1190 | 193.2 | 0.32 | 841.2 | 12947 | -95.9% | 5.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 495.0 | 0.17 | 690.6 | 1.01 | 0.72 | 6.34 | +39.5% | 270.7 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 53553 | 13590 | 104682 | 23212 | 19019 | 169108 | +95.5% | 2.69 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `multi`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P+ | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs P | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs P | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P vs M | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| N vs H3 | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c2_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c2_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c2_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c4_q1_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c4_q1_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c4_q2_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | c4_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | write_p50_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | write_p99_us | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | commits_per_s | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | view_state_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | pss_mib | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |
| P+ vs H3 | pss_growth_bytes_per_key_read | — | — | — | — | — | — | — | — | REFUSED (the point recorded a failure) |

## `multi`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 251.8 | 33.0 | 685.3 | 48.8 | 41.7 | 201.7 | +172.1% | 10.4 | **N better** |
| N vs P+ | q1_p99_us | 2252 | 1553 | 3996 | 1135 | 1360 | 1573 | +77.5% | 1.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 870804 | 75707 | 535.3 | 36.0 | 53533 | 137173 | -99.9% | 16.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 227.1 | 60.3 | 345.1 | 45.6 | 53.5 | 27.4 | +52.0% | 2.21 | no difference |
| N vs P+ | q4_p50_us | 198957 | 5063 | 76846 | 4084 | 4600 | 6799 | -61.4% | 26.5 | **P+ better** |
| N vs P+ | q5_p50_us | 367384 | 195031 | 472751 | 369593 | 295497 | 10904 | +28.7% | 0.36 | no difference |
| N vs P+ | q6_p50_us | 1243697 | 28754 | 1651067 | 174730 | 125215 | 124082 | +32.8% | 3.25 | **N better** |
| N vs P+ | c2_q1_p50_us | 118.8 | 27.8 | 634.1 | 96.2 | 70.8 | 196.0 | +433.7% | 7.28 | **N better** |
| N vs P+ | c2_q1_p99_us | 558.4 | 95.8 | 4519 | 491.2 | 353.9 | 2153 | +709.4% | 11.2 | **N better** |
| N vs P+ | c2_q2_p50_us | 805816 | 30232 | 542.6 | 23.7 | 21377 | 79114 | -99.9% | 37.7 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 116.5 | 17.4 | 1652 | 101.0 | 72.5 | 237.6 | +1318.2% | 21.2 | **N better** |
| N vs P+ | c4_q1_p99_us | 5911 | 273.2 | 8218 | 576.4 | 451.1 | 105.1 | +39.0% | 5.11 | **N better** |
| N vs P+ | c4_q2_p50_us | 1332836 | 53560 | 1548 | 118.7 | 37873 | 38308 | -99.9% | 35.2 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 3102202 | 47052 | 8333 | 1495 | 33288 | 23789 | -99.7% | 92.9 | **P+ better** |
| N vs P+ | write_p50_us | 905.9 | 32.3 | 1146 | 140.7 | 102.1 | 369.7 | +26.6% | 2.36 | BELOW FLOOR |
| N vs P+ | write_p99_us | 2664 | 844.6 | 3711 | 543.7 | 710.3 | 4955 | +39.3% | 1.47 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 861.7 | 92.7 | 693.7 | 60.0 | 78.1 | 183.2 | -19.5% | 2.15 | BELOW FLOOR |
| N vs P+ | view_state_bytes_per_key_read | 6661 | 1604 | 4538 | 1097 | 1374 | 21759 | -31.9% | 1.55 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 3031 | 0.41 | 493.6 | 0.20 | 0.32 | 170.2 | -83.7% | 7834 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 713075 | 174853 | 53214 | 12963 | 123979 | 1770697 | -92.5% | 5.32 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 251.8 | 33.0 | 118.9 | 6.44 | 23.8 | 72.3 | -52.8% | 5.60 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q1_p99_us | 2252 | 1553 | 1980 | 213.0 | 1109 | 333.6 | -12.1% | 0.25 | BELOW FLOOR |
| N vs P | q2_p50_us | 870804 | 75707 | 231.1 | 15.4 | 53533 | 137173 | -100.0% | 16.3 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 227.1 | 60.3 | 293.6 | 88.7 | 75.9 | 47.3 | +29.3% | 0.88 | no difference |
| N vs P | q4_p50_us | 198957 | 5063 | 6592 | 376.0 | 3590 | 624.6 | -96.7% | 53.6 | **P better** |
| N vs P | q5_p50_us | 367384 | 195031 | 223.2 | 38.1 | 137908 | 6708 | -99.9% | 2.66 | no difference |
| N vs P | q6_p50_us | 1243697 | 28754 | 34557 | 1571 | 20363 | 47442 | -97.2% | 59.4 | **P better** |
| N vs P | c2_q1_p50_us | 118.8 | 27.8 | 72.9 | 26.3 | 27.0 | 13.4 | -38.7% | 1.70 | no difference |
| N vs P | c2_q1_p99_us | 558.4 | 95.8 | 1973 | 1017 | 722.0 | 390.6 | +253.3% | 1.96 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 805816 | 30232 | 177.6 | 44.7 | 21377 | 79114 | -100.0% | 37.7 | **P better** |
| N vs P | c4_q1_p50_us | 116.5 | 17.4 | 118.2 | 26.1 | 22.2 | 31.1 | +1.5% | 0.08 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 5911 | 273.2 | 3308 | 866.8 | 642.6 | 488.4 | -44.0% | 4.05 | **P better** |
| N vs P | c4_q2_p50_us | 1332836 | 53560 | 299.2 | 28.2 | 37873 | 38307 | -100.0% | 35.2 | **P better** |
| N vs P | c4_q2_p99_us | 3102202 | 47052 | 8466 | 4502 | 33423 | 23850 | -99.7% | 92.6 | **P better** |
| N vs P | write_p50_us | 905.9 | 32.3 | 2166352 | 109167 | 77193 | 40585 | +239044.9% | 28.1 | **N better** |
| N vs P | write_p99_us | 2664 | 844.6 | 2524645 | 109773 | 77623 | 235191 | +94678.4% | 32.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 861.7 | 92.7 | 0.46 | 0.02 | 65.6 | 100.9 | -99.9% | 13.1 | **N better** |
| N vs P | view_state_bytes_per_key_read | 6661 | 1604 | 3724 | 912.4 | 1305 | 20724 | -44.1% | 2.25 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 3031 | 0.41 | 488.3 | 1.07 | 0.81 | 170.2 | -83.9% | 3143 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 713075 | 174853 | 53033 | 12608 | 123961 | 1771280 | -92.6% | 5.32 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 685.3 | 48.8 | 118.9 | 6.44 | 34.8 | 211.9 | -82.7% | 16.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p99_us | 3996 | 1135 | 1980 | 213.0 | 816.4 | 1547 | -50.5% | 2.47 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 535.3 | 36.0 | 231.1 | 15.4 | 27.7 | 236.2 | -56.8% | 11.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 345.1 | 45.6 | 293.6 | 88.7 | 70.6 | 45.5 | -14.9% | 0.73 | no difference |
| P+ vs P | q4_p50_us | 76846 | 4084 | 6592 | 376.0 | 2900 | 6783 | -91.4% | 24.2 | **P better** |
| P+ vs P | q5_p50_us | 472751 | 369593 | 223.2 | 38.1 | 261342 | 8597 | -100.0% | 1.81 | no difference |
| P+ vs P | q6_p50_us | 1651067 | 174730 | 34557 | 1571 | 123558 | 114684 | -97.9% | 13.1 | **P better** |
| P+ vs P | c2_q1_p50_us | 634.1 | 96.2 | 72.9 | 26.3 | 70.5 | 195.9 | -88.5% | 7.96 | **P better** |
| P+ vs P | c2_q1_p99_us | 4519 | 491.2 | 1973 | 1017 | 798.3 | 2179 | -56.4% | 3.19 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 542.6 | 23.7 | 177.6 | 44.7 | 35.8 | 47.7 | -67.3% | 10.2 | **P better** |
| P+ vs P | c4_q1_p50_us | 1652 | 101.0 | 118.2 | 26.1 | 73.8 | 239.1 | -92.8% | 20.8 | **P better** |
| P+ vs P | c4_q1_p99_us | 8218 | 576.4 | 3308 | 866.8 | 736.1 | 477.1 | -59.7% | 6.67 | **P better** |
| P+ vs P | c4_q2_p50_us | 1548 | 118.7 | 299.2 | 28.2 | 86.3 | 373.6 | -80.7% | 14.5 | **P better** |
| P+ vs P | c4_q2_p99_us | 8333 | 1495 | 8466 | 4502 | 3354 | 2496 | +1.6% | 0.04 | BELOW FLOOR |
| P+ vs P | write_p50_us | 1146 | 140.7 | 2166352 | 109167 | 77193 | 40583 | +188862.8% | 28.0 | **P+ better** |
| P+ vs P | write_p99_us | 3711 | 543.7 | 2524645 | 109773 | 77622 | 235138 | +67936.7% | 32.5 | **P+ better** |
| P+ vs P | commits_per_s | 693.7 | 60.0 | 0.46 | 0.02 | 42.4 | 152.9 | -99.9% | 16.3 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 4538 | 1097 | 3724 | 912.4 | 1009 | 15943 | -17.9% | 0.81 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 493.6 | 0.20 | 488.3 | 1.07 | 0.77 | 0.83 | -1.1% | 7.00 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 53214 | 12963 | 53033 | 12608 | 12787 | 198897 | -0.3% | 0.01 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 118.9 | 6.44 | 115.2 | 6.99 | 6.72 | 72.5 | -3.1% | 0.55 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q1_p99_us | 1980 | 213.0 | 2022 | 264.7 | 240.2 | 442.1 | +2.2% | 0.18 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 231.1 | 15.4 | 252.4 | 15.1 | 15.3 | 116.4 | +9.2% | 1.40 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 293.6 | 88.7 | 275.3 | 38.5 | 68.4 | 70.9 | -6.3% | 0.27 | BELOW FLOOR |
| P vs M | q4_p50_us | 6592 | 376.0 | 6332 | 219.1 | 307.7 | 435.0 | -3.9% | 0.84 | BELOW FLOOR |
| P vs M | q5_p50_us | 223.2 | 38.1 | 200.2 | 32.8 | 35.5 | 64.3 | -10.3% | 0.65 | BELOW FLOOR |
| P vs M | q6_p50_us | 34557 | 1571 | 35844 | 870.8 | 1270 | 2972 | +3.7% | 1.01 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 72.9 | 26.3 | 38.6 | 4.10 | 18.8 | 19.3 | -47.1% | 1.83 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1973 | 1017 | 1118 | 120.5 | 723.8 | 364.1 | -43.4% | 1.18 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 177.6 | 44.7 | 129.9 | 19.7 | 34.6 | 8.82 | -26.8% | 1.38 | no difference |
| P vs M | c4_q1_p50_us | 118.2 | 26.1 | 118.1 | 49.2 | 39.4 | 45.5 | -0.1% | 0.00 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 3308 | 866.8 | 3516 | 573.7 | 735.0 | 1716 | +6.3% | 0.28 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 299.2 | 28.2 | 328.4 | 88.9 | 65.9 | 33.4 | +9.7% | 0.44 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 8466 | 4502 | 9034 | 4135 | 4322 | 2277 | +6.7% | 0.13 | BELOW FLOOR |
| P vs M | write_p50_us | 2166352 | 109167 | 2147442 | 103877 | 106555 | 45838 | -0.9% | 0.18 | BELOW FLOOR |
| P vs M | write_p99_us | 2524645 | 109773 | 2497849 | 94894 | 102603 | 251621 | -1.1% | 0.26 | BELOW FLOOR |
| P vs M | commits_per_s | 0.46 | 0.02 | 0.46 | 0.02 | 0.02 | 0.00 | +1.1% | 0.24 | no difference |
| P vs M | view_state_bytes_per_key_read | 3724 | 912.4 | 3724 | 912.4 | 912.4 | 14499 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 488.3 | 1.07 | 488.9 | 0.84 | 0.96 | 0.27 | +0.1% | 0.62 | no difference |
| P vs M | pss_growth_bytes_per_key_read | 53033 | 12608 | 53151 | 12878 | 12744 | 202069 | +0.2% | 0.01 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 251.8 | 33.0 | 319.5 | 32.7 | 32.8 | 32.5 | +26.8% | 2.06 | no difference |
| N vs H3 | q1_p99_us | 2252 | 1553 | 2873 | 776.3 | 1228 | 623.1 | +27.6% | 0.51 | BELOW FLOOR |
| N vs H3 | q2_p50_us | 870804 | 75707 | 306.2 | 25.5 | 53533 | 137173 | -100.0% | 16.3 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 227.1 | 60.3 | 362.1 | 42.5 | 52.2 | 25.5 | +59.4% | 2.59 | no difference |
| N vs H3 | q4_p50_us | 198957 | 5063 | 80054 | 7072 | 6150 | 2959 | -59.8% | 19.3 | **H3 better** |
| N vs H3 | q5_p50_us | 367384 | 195031 | 461715 | 350931 | 283892 | 16723 | +25.7% | 0.33 | no difference |
| N vs H3 | q6_p50_us | 1243697 | 28754 | 1621101 | 100393 | 73843 | 100861 | +30.3% | 5.11 | **N better** |
| N vs H3 | c2_q1_p50_us | 118.8 | 27.8 | 229.7 | 14.9 | 22.3 | 10.8 | +93.3% | 4.97 | **N better** |
| N vs H3 | c2_q1_p99_us | 558.4 | 95.8 | 3595 | 1699 | 1203 | 10736 | +543.9% | 2.52 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 805816 | 30232 | 243.3 | 50.3 | 21377 | 79114 | -100.0% | 37.7 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 116.5 | 17.4 | 265.9 | 31.3 | 25.3 | 108.6 | +128.3% | 5.90 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 5911 | 273.2 | 4441 | 966.5 | 710.2 | 10330 | -24.9% | 2.07 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 1332836 | 53560 | 291.5 | 50.8 | 37873 | 38307 | -100.0% | 35.2 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 3102202 | 47052 | 4820 | 1714 | 33293 | 24575 | -99.8% | 93.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 905.9 | 32.3 | 1755 | 102.5 | 76.0 | 382.4 | +93.8% | 11.2 | **N better** |
| N vs H3 | write_p99_us | 2664 | 844.6 | 5654 | 1839 | 1431 | 6642 | +112.2% | 2.09 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 861.7 | 92.7 | 476.4 | 92.6 | 92.7 | 101.2 | -44.7% | 4.16 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 6661 | 1604 | 195.1 | 1.48 | 1134 | 18011 | -97.1% | 5.70 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 3031 | 0.41 | 592.5 | 0.36 | 0.39 | 170.3 | -80.5% | 6275 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 713075 | 174853 | 80716 | 19766 | 124427 | 1769120 | -88.7% | 5.08 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 685.3 | 48.8 | 319.5 | 32.7 | 41.5 | 201.8 | -53.4% | 8.81 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3996 | 1135 | 2873 | 776.3 | 972.2 | 1634 | -28.1% | 1.16 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 535.3 | 36.0 | 306.2 | 25.5 | 31.2 | 235.4 | -42.8% | 7.34 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 345.1 | 45.6 | 362.1 | 42.5 | 44.1 | 21.9 | +4.9% | 0.39 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 76846 | 4084 | 80054 | 7072 | 5775 | 7374 | +4.2% | 0.56 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 472751 | 369593 | 461715 | 350931 | 360383 | 17566 | -2.3% | 0.03 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 1651067 | 174730 | 1621101 | 100393 | 142494 | 145171 | -1.8% | 0.21 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 634.1 | 96.2 | 229.7 | 14.9 | 68.9 | 195.7 | -63.8% | 5.87 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4519 | 491.2 | 3595 | 1699 | 1250 | 10948 | -20.4% | 0.74 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 542.6 | 23.7 | 243.3 | 50.3 | 39.3 | 47.1 | -55.2% | 7.61 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1652 | 101.0 | 265.9 | 31.3 | 74.8 | 260.8 | -83.9% | 18.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 8218 | 576.4 | 4441 | 966.5 | 795.7 | 10329 | -46.0% | 4.75 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1548 | 118.7 | 291.5 | 50.8 | 91.3 | 377.8 | -81.2% | 13.8 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8333 | 1495 | 4820 | 1714 | 1609 | 6428 | -42.2% | 2.18 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1146 | 140.7 | 1755 | 102.5 | 123.1 | 113.1 | +53.1% | 4.95 | **P+ better** |
| P+ vs H3 | write_p99_us | 3711 | 543.7 | 5654 | 1839 | 1356 | 4425 | +52.4% | 1.43 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 693.7 | 60.0 | 476.4 | 92.6 | 78.0 | 153.1 | -31.3% | 2.79 | no difference |
| P+ vs H3 | view_state_bytes_per_key_read | 4538 | 1097 | 195.1 | 1.48 | 776.0 | 12209 | -95.7% | 5.60 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 493.6 | 0.20 | 592.5 | 0.36 | 0.29 | 5.45 | +20.0% | 340.2 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 53214 | 12963 | 80716 | 19766 | 16714 | 178640 | +51.7% | 1.65 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 100.6 | 5.95 | 601.5 | 42.9 | 30.6 | 46.6 | +497.9% | 16.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 376.8 | 53.1 | 1874 | 845.7 | 599.2 | 176.0 | +397.3% | 2.50 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 134.7 | 10.1 | 461.1 | 34.0 | 25.1 | 21.8 | +242.3% | 13.0 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 119.9 | 7.28 | 265.8 | 29.5 | 21.5 | 78.8 | +121.6% | 6.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | 2128 | 30.8 | 1304 | 65.9 | 51.5 | 55.1 | -38.7% | 16.0 | **P+ better** |
| N vs P+ | q5_p50_us | 2346 | 44.6 | 3057 | 74.1 | 61.2 | 112.7 | +30.3% | 11.6 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 145.2 | 12.6 | 697.5 | 70.5 | 50.7 | 24.9 | +380.3% | 10.9 | **N better** |
| N vs P+ | c2_q1_p99_us | 619.7 | 172.2 | 5405 | 798.5 | 577.6 | 1526 | +772.2% | 8.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 177.2 | 17.2 | 524.4 | 44.4 | 33.6 | 49.0 | +195.9% | 10.3 | **N better** |
| N vs P+ | c4_q1_p50_us | 219.8 | 12.5 | 1819 | 169.9 | 120.5 | 39.4 | +727.5% | 13.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 996.3 | 139.5 | 16899 | 12083 | 8544 | 9638 | +1596.2% | 1.86 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 240.7 | 24.6 | 1478 | 99.8 | 72.7 | 149.9 | +513.9% | 17.0 | **N better** |
| N vs P+ | c4_q2_p99_us | 1055 | 196.5 | 10497 | 6018 | 4257 | 4572 | +894.8% | 2.22 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 482.6 | 18.2 | 835.0 | 121.7 | 87.0 | 108.0 | +73.0% | 4.05 | **N better** |
| N vs P+ | write_p99_us | 851.0 | 94.0 | 3164 | 621.0 | 444.1 | 801.9 | +271.8% | 5.21 | **N better** |
| N vs P+ | commits_per_s | 1892 | 139.0 | 1005 | 71.2 | 110.4 | 275.1 | -46.9% | 8.04 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 328.8 | 6.12 | 986.9 | 50.8 | 36.2 | 289.3 | +200.1% | 18.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 10.3 | 0.03 | 43.9 | 0.01 | 0.03 | 1.16 | +325.4% | 1330 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 5052 | 144.7 | -7838 | 281.6 | 223.9 | 6361 | -255.1% | 57.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 100.6 | 5.95 | 94.3 | 13.5 | 10.4 | 45.2 | -6.3% | 0.60 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q1_p99_us | 376.8 | 53.1 | 603.9 | 203.8 | 148.9 | 147.8 | +60.3% | 1.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 134.7 | 10.1 | 171.5 | 18.7 | 15.0 | 41.3 | +27.3% | 2.45 | BELOW FLOOR |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 119.9 | 7.28 | 208.8 | 29.9 | 21.8 | 82.6 | +74.1% | 4.08 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q4_p50_us | 2128 | 30.8 | 211.7 | 36.6 | 33.8 | 124.4 | -90.1% | 56.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q5_p50_us | 2346 | 44.6 | 148.2 | 33.1 | 39.3 | 112.5 | -93.7% | 56.0 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 145.2 | 12.6 | 93.6 | 11.3 | 11.9 | 7.17 | -35.6% | 4.33 | **P better** |
| N vs P | c2_q1_p99_us | 619.7 | 172.2 | 1807 | 248.4 | 213.8 | 138.2 | +191.6% | 5.55 | **N better** |
| N vs P | c2_q2_p50_us | 177.2 | 17.2 | 179.8 | 26.9 | 22.6 | 19.7 | +1.5% | 0.12 | BELOW FLOOR |
| N vs P | c4_q1_p50_us | 219.8 | 12.5 | 169.5 | 31.1 | 23.7 | 45.6 | -22.9% | 2.12 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 996.3 | 139.5 | 2438 | 231.5 | 191.1 | 290.6 | +144.7% | 7.54 | **N better** |
| N vs P | c4_q2_p50_us | 240.7 | 24.6 | 291.7 | 27.9 | 26.3 | 35.7 | +21.2% | 1.94 | no difference |
| N vs P | c4_q2_p99_us | 1055 | 196.5 | 3006 | 804.7 | 585.7 | 520.0 | +184.9% | 3.33 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 482.6 | 18.2 | 12777 | 379.3 | 268.5 | 82.1 | +2547.4% | 45.8 | **N better** |
| N vs P | write_p99_us | 851.0 | 94.0 | 17918 | 1676 | 1187 | 927.0 | +2005.6% | 14.4 | **N better** |
| N vs P | commits_per_s | 1892 | 139.0 | 74.0 | 4.09 | 98.3 | 234.2 | -96.1% | 18.5 | **N better** |
| N vs P | view_state_bytes_per_key_read | 328.8 | 6.12 | 254.1 | 7.88 | 7.05 | 214.3 | -22.7% | 10.6 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 10.3 | 0.03 | 44.2 | 0.02 | 0.03 | 1.15 | +327.7% | 1242 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 5052 | 144.7 | -5485 | 175.0 | 160.5 | 4325 | -208.6% | 65.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 601.5 | 42.9 | 94.3 | 13.5 | 31.8 | 17.6 | -84.3% | 15.9 | **P better** |
| P+ vs P | q1_p99_us | 1874 | 845.7 | 603.9 | 203.8 | 615.1 | 95.6 | -67.8% | 2.06 | no difference |
| P+ vs P | q2_p50_us | 461.1 | 34.0 | 171.5 | 18.7 | 27.5 | 46.6 | -62.8% | 10.5 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 265.8 | 29.5 | 208.8 | 29.9 | 29.7 | 36.7 | -21.4% | 1.92 | no difference |
| P+ vs P | q4_p50_us | 1304 | 65.9 | 211.7 | 36.6 | 53.3 | 119.7 | -83.8% | 20.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q5_p50_us | 3057 | 74.1 | 148.2 | 33.1 | 57.4 | 12.8 | -95.2% | 50.7 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 697.5 | 70.5 | 93.6 | 11.3 | 50.5 | 25.6 | -86.6% | 12.0 | **P better** |
| P+ vs P | c2_q1_p99_us | 5405 | 798.5 | 1807 | 248.4 | 591.3 | 1520 | -66.6% | 6.08 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 524.4 | 44.4 | 179.8 | 26.9 | 36.7 | 47.1 | -65.7% | 9.39 | **P better** |
| P+ vs P | c4_q1_p50_us | 1819 | 169.9 | 169.5 | 31.1 | 122.1 | 54.4 | -90.7% | 13.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 16899 | 12083 | 2438 | 231.5 | 8545 | 9642 | -85.6% | 1.69 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1478 | 99.8 | 291.7 | 27.9 | 73.3 | 147.4 | -80.3% | 16.2 | **P better** |
| P+ vs P | c4_q2_p99_us | 10497 | 6018 | 3006 | 804.7 | 4293 | 4566 | -71.4% | 1.74 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 835.0 | 121.7 | 12777 | 379.3 | 281.7 | 73.8 | +1430.2% | 42.4 | **P+ better** |
| P+ vs P | write_p99_us | 3164 | 621.0 | 17918 | 1676 | 1264 | 1195 | +466.4% | 11.7 | **P+ better** |
| P+ vs P | commits_per_s | 1005 | 71.2 | 74.0 | 4.09 | 50.4 | 144.7 | -92.6% | 18.5 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 986.9 | 50.8 | 254.1 | 7.88 | 36.3 | 268.9 | -74.3% | 20.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | pss_mib | 43.9 | 0.01 | 44.2 | 0.02 | 0.02 | 0.10 | +0.5% | 13.9 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -7838 | 281.6 | -5485 | 175.0 | 234.4 | 7615 | -30.0% | 10.0 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 94.3 | 13.5 | 96.7 | 11.0 | 12.3 | 10.4 | +2.6% | 0.20 | BELOW FLOOR |
| P vs M | q1_p99_us | 603.9 | 203.8 | 546.9 | 182.3 | 193.4 | 348.5 | -9.4% | 0.29 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 171.5 | 18.7 | 184.4 | 23.5 | 21.2 | 42.1 | +7.5% | 0.61 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 208.8 | 29.9 | 229.1 | 28.4 | 29.2 | 60.5 | +9.8% | 0.70 | BELOW FLOOR |
| P vs M | q4_p50_us | 211.7 | 36.6 | 241.9 | 48.8 | 43.1 | 115.7 | +14.3% | 0.70 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q5_p50_us | 148.2 | 33.1 | 140.9 | 13.5 | 25.2 | 7.29 | -4.9% | 0.29 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 93.6 | 11.3 | 89.8 | 4.50 | 8.58 | 6.69 | -4.0% | 0.44 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1807 | 248.4 | 1438 | 599.9 | 459.1 | 416.6 | -20.4% | 0.80 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 179.8 | 26.9 | 175.5 | 28.8 | 27.9 | 24.0 | -2.4% | 0.16 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 169.5 | 31.1 | 151.5 | 57.0 | 45.9 | 45.6 | -10.6% | 0.39 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2438 | 231.5 | 2076 | 347.8 | 295.4 | 341.0 | -14.9% | 1.23 | no difference |
| P vs M | c4_q2_p50_us | 291.7 | 27.9 | 243.4 | 16.1 | 22.8 | 36.8 | -16.6% | 2.12 | no difference |
| P vs M | c4_q2_p99_us | 3006 | 804.7 | 2635 | 667.2 | 739.2 | 351.6 | -12.3% | 0.50 | no difference |
| P vs M | write_p50_us | 12777 | 379.3 | 12932 | 614.8 | 510.8 | 287.3 | +1.2% | 0.30 | BELOW FLOOR |
| P vs M | write_p99_us | 17918 | 1676 | 18838 | 2335 | 2033 | 1255 | +5.1% | 0.45 | BELOW FLOOR |
| P vs M | commits_per_s | 74.0 | 4.09 | 72.9 | 4.50 | 4.30 | 8.44 | -1.6% | 0.28 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 254.1 | 7.88 | 255.6 | 2.69 | 5.89 | 185.9 | +0.6% | 0.25 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.2 | 0.02 | 44.2 | 0.01 | 0.01 | 0.01 | +0.1% | 2.90 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -5485 | 175.0 | -3732 | 122.0 | 150.8 | 5223 | -32.0% | 11.6 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 100.6 | 5.95 | 233.3 | 20.2 | 14.9 | 56.4 | +131.9% | 8.89 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 376.8 | 53.1 | 773.4 | 284.7 | 204.8 | 164.8 | +105.3% | 1.94 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 134.7 | 10.1 | 239.7 | 35.1 | 25.9 | 49.4 | +78.0% | 4.06 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 119.9 | 7.28 | 308.7 | 63.3 | 45.1 | 105.3 | +157.4% | 4.19 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | 2128 | 30.8 | 1352 | 43.7 | 37.8 | 47.2 | -36.5% | 20.6 | **H3 better** |
| N vs H3 | q5_p50_us | 2346 | 44.6 | 3229 | 251.2 | 180.4 | 184.2 | +37.7% | 4.90 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 145.2 | 12.6 | 195.3 | 48.5 | 35.4 | 5.33 | +34.5% | 1.42 | no difference |
| N vs H3 | c2_q1_p99_us | 619.7 | 172.2 | 826.1 | 201.5 | 187.4 | 880.2 | +33.3% | 1.10 | BELOW FLOOR |
| N vs H3 | c2_q2_p50_us | 177.2 | 17.2 | 208.1 | 47.4 | 35.6 | 17.0 | +17.4% | 0.87 | no difference |
| N vs H3 | c4_q1_p50_us | 219.8 | 12.5 | 319.4 | 35.9 | 26.9 | 53.6 | +45.3% | 3.70 | **N better** |
| N vs H3 | c4_q1_p99_us | 996.3 | 139.5 | 1465 | 382.6 | 288.0 | 149.6 | +47.1% | 1.63 | no difference |
| N vs H3 | c4_q2_p50_us | 240.7 | 24.6 | 323.6 | 38.2 | 32.1 | 31.7 | +34.4% | 2.58 | no difference |
| N vs H3 | c4_q2_p99_us | 1055 | 196.5 | 1354 | 326.5 | 269.4 | 1182 | +28.3% | 1.11 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 482.6 | 18.2 | 1131 | 96.4 | 69.4 | 108.3 | +134.3% | 9.34 | **N better** |
| N vs H3 | write_p99_us | 851.0 | 94.0 | 4743 | 1305 | 924.9 | 1558 | +457.4% | 4.21 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | commits_per_s | 1892 | 139.0 | 663.9 | 94.8 | 118.9 | 236.5 | -64.9% | 10.3 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 328.8 | 6.12 | 110.0 | 0.79 | 4.36 | 169.3 | -66.5% | 50.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 10.3 | 0.03 | 70.6 | 0.10 | 0.07 | 1.25 | +583.9% | 823.2 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 5052 | 144.7 | 17146 | 453.1 | 336.3 | 3800 | +239.4% | 36.0 | **N better** |
| P+ vs H3 | q1_p50_us | 601.5 | 42.9 | 233.3 | 20.2 | 33.5 | 38.1 | -61.2% | 11.0 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1874 | 845.7 | 773.4 | 284.7 | 631.0 | 120.2 | -58.7% | 1.74 | no difference |
| P+ vs H3 | q2_p50_us | 461.1 | 34.0 | 239.7 | 35.1 | 34.6 | 53.9 | -48.0% | 6.40 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 265.8 | 29.5 | 308.7 | 63.3 | 49.4 | 74.8 | +16.1% | 0.87 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1304 | 65.9 | 1352 | 43.7 | 55.9 | 33.0 | +3.6% | 0.85 | no difference |
| P+ vs H3 | q5_p50_us | 3057 | 74.1 | 3229 | 251.2 | 185.2 | 146.4 | +5.6% | 0.93 | no difference |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 697.5 | 70.5 | 195.3 | 48.5 | 60.5 | 25.1 | -72.0% | 8.30 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5405 | 798.5 | 826.1 | 201.5 | 582.3 | 1751 | -84.7% | 7.86 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p50_us | 524.4 | 44.4 | 208.1 | 47.4 | 45.9 | 46.0 | -60.3% | 6.89 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1819 | 169.9 | 319.4 | 35.9 | 122.8 | 61.3 | -82.4% | 12.2 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 16899 | 12083 | 1465 | 382.6 | 8548 | 9638 | -91.3% | 1.81 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1478 | 99.8 | 323.6 | 38.2 | 75.6 | 146.5 | -78.1% | 15.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 10497 | 6018 | 1354 | 326.5 | 4261 | 4688 | -87.1% | 2.15 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 835.0 | 121.7 | 1131 | 96.4 | 109.8 | 102.1 | +35.4% | 2.69 | no difference |
| P+ vs H3 | write_p99_us | 3164 | 621.0 | 4743 | 1305 | 1022 | 1731 | +49.9% | 1.55 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 1005 | 71.2 | 663.9 | 94.8 | 83.8 | 148.4 | -33.9% | 4.06 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 986.9 | 50.8 | 110.0 | 0.79 | 35.9 | 234.7 | -88.9% | 24.4 | **H3 better** |
| P+ vs H3 | pss_mib | 43.9 | 0.01 | 70.6 | 0.10 | 0.07 | 0.50 | +60.8% | 381.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -7838 | 281.6 | 17146 | 453.1 | 377.2 | 7329 | -318.8% | 66.2 | REFUSED (warm-up MAD above 15% of the median on P+) |

## `single`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 111.9 | 7.95 | 594.8 | 26.9 | 19.9 | 57.0 | +431.7% | 24.3 | **N better** |
| N vs P+ | q1_p99_us | 449.5 | 172.7 | 3231 | 940.6 | 676.2 | 1171 | +618.9% | 4.11 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 132.0 | 10.2 | 446.8 | 15.1 | 12.9 | 114.7 | +238.5% | 24.5 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 134.3 | 4.96 | 249.4 | 24.0 | 17.3 | 91.1 | +85.7% | 6.64 | **N better** |
| N vs P+ | q4_p50_us | 2176 | 57.9 | 1319 | 32.6 | 47.0 | 73.0 | -39.4% | 18.2 | **P+ better** |
| N vs P+ | q5_p50_us | 2372 | 29.2 | 3085 | 62.8 | 49.0 | 436.0 | +30.0% | 14.5 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 141.1 | 11.8 | 647.2 | 49.3 | 35.9 | 54.8 | +358.7% | 14.1 | **N better** |
| N vs P+ | c2_q1_p99_us | 665.9 | 106.3 | 5170 | 540.4 | 389.4 | 856.2 | +676.3% | 11.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 162.3 | 13.7 | 526.1 | 21.0 | 17.7 | 60.5 | +224.2% | 20.5 | **N better** |
| N vs P+ | c4_q1_p50_us | 235.2 | 33.8 | 1796 | 104.6 | 77.7 | 108.2 | +663.6% | 20.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 954.5 | 110.1 | 8963 | 2295 | 1625 | 479.0 | +839.0% | 4.93 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 274.1 | 31.1 | 1578 | 127.6 | 92.8 | 399.2 | +475.9% | 14.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1495 | 333.2 | 5763 | 1917 | 1376 | 2287 | +285.5% | 3.10 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 452.3 | 21.7 | 828.9 | 52.5 | 40.1 | 30.4 | +83.3% | 9.38 | **N better** |
| N vs P+ | write_p99_us | 1002 | 185.6 | 2572 | 160.8 | 173.7 | 438.5 | +156.7% | 9.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1942 | 96.1 | 1045 | 46.4 | 75.4 | 203.6 | -46.2% | 11.9 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 333.6 | 8.16 | 993.1 | 53.5 | 38.3 | 282.1 | +197.7% | 17.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 9.98 | 0.11 | 43.9 | 0.01 | 0.08 | 0.09 | +340.4% | 422.6 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4785 | 19.0 | -2676 | 114.8 | 82.3 | 2773 | -155.9% | 90.7 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 111.9 | 7.95 | 96.1 | 5.43 | 6.81 | 6.37 | -14.1% | 2.31 | no difference |
| N vs P | q1_p99_us | 449.5 | 172.7 | 706.1 | 296.5 | 242.6 | 662.8 | +57.1% | 1.06 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 132.0 | 10.2 | 182.4 | 16.6 | 13.8 | 14.3 | +38.2% | 3.67 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 134.3 | 4.96 | 215.8 | 16.6 | 12.2 | 33.2 | +60.7% | 6.66 | **N better** |
| N vs P | q4_p50_us | 2176 | 57.9 | 180.9 | 10.1 | 41.6 | 71.7 | -91.7% | 48.0 | **P better** |
| N vs P | q5_p50_us | 2372 | 29.2 | 136.1 | 20.7 | 25.3 | 109.2 | -94.3% | 88.3 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 141.1 | 11.8 | 99.3 | 11.5 | 11.7 | 32.5 | -29.6% | 3.58 | **P better** |
| N vs P | c2_q1_p99_us | 665.9 | 106.3 | 1831 | 194.0 | 156.5 | 581.6 | +175.0% | 7.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 162.3 | 13.7 | 182.4 | 31.3 | 24.1 | 40.5 | +12.4% | 0.83 | BELOW FLOOR |
| N vs P | c4_q1_p50_us | 235.2 | 33.8 | 150.9 | 36.2 | 35.0 | 116.0 | -35.9% | 2.41 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 954.5 | 110.1 | 2280 | 283.6 | 215.2 | 849.3 | +138.9% | 6.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 274.1 | 31.1 | 247.7 | 21.5 | 26.8 | 186.3 | -9.6% | 0.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 1495 | 333.2 | 2361 | 1058 | 784.4 | 1085 | +57.9% | 1.10 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 452.3 | 21.7 | 12935 | 162.6 | 116.0 | 826.1 | +2759.9% | 107.6 | **N better** |
| N vs P | write_p99_us | 1002 | 185.6 | 18740 | 1287 | 919.2 | 1138 | +1770.1% | 19.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1942 | 96.1 | 72.0 | 2.48 | 68.0 | 170.9 | -96.3% | 27.5 | **N better** |
| N vs P | view_state_bytes_per_key_read | 333.6 | 8.16 | 256.1 | 1.99 | 5.94 | 205.9 | -23.2% | 13.0 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 9.98 | 0.11 | 44.1 | 0.01 | 0.08 | 0.10 | +342.2% | 425.3 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 4785 | 19.0 | -2469 | 103.2 | 74.2 | 2657 | -151.6% | 97.7 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 594.8 | 26.9 | 96.1 | 5.43 | 19.4 | 56.9 | -83.8% | 25.7 | **P better** |
| P+ vs P | q1_p99_us | 3231 | 940.6 | 706.1 | 296.5 | 697.4 | 968.7 | -78.1% | 3.62 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 446.8 | 15.1 | 182.4 | 16.6 | 15.8 | 115.6 | -59.2% | 16.7 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 249.4 | 24.0 | 215.8 | 16.6 | 20.6 | 96.4 | -13.5% | 1.63 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1319 | 32.6 | 180.9 | 10.1 | 24.1 | 20.1 | -86.3% | 47.2 | **P better** |
| P+ vs P | q5_p50_us | 3085 | 62.8 | 136.1 | 20.7 | 46.8 | 422.8 | -95.6% | 63.0 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 647.2 | 49.3 | 99.3 | 11.5 | 35.8 | 44.7 | -84.7% | 15.3 | **P better** |
| P+ vs P | c2_q1_p99_us | 5170 | 540.4 | 1831 | 194.0 | 406.0 | 630.3 | -64.6% | 8.22 | **P better** |
| P+ vs P | c2_q2_p50_us | 526.1 | 21.0 | 182.4 | 31.3 | 26.6 | 51.5 | -65.3% | 12.9 | **P better** |
| P+ vs P | c4_q1_p50_us | 1796 | 104.6 | 150.9 | 36.2 | 78.3 | 54.6 | -91.6% | 21.0 | **P better** |
| P+ vs P | c4_q1_p99_us | 8963 | 2295 | 2280 | 283.6 | 1635 | 837.0 | -74.6% | 4.09 | **P better** |
| P+ vs P | c4_q2_p50_us | 1578 | 127.6 | 247.7 | 21.5 | 91.5 | 353.1 | -84.3% | 14.5 | **P better** |
| P+ vs P | c4_q2_p99_us | 5763 | 1917 | 2361 | 1058 | 1548 | 2066 | -59.0% | 2.20 | no difference |
| P+ vs P | write_p50_us | 828.9 | 52.5 | 12935 | 162.6 | 120.8 | 826.5 | +1460.6% | 100.2 | **P+ better** |
| P+ vs P | write_p99_us | 2572 | 160.8 | 18740 | 1287 | 916.9 | 1079 | +628.6% | 17.6 | **P+ better** |
| P+ vs P | commits_per_s | 1045 | 46.4 | 72.0 | 2.48 | 32.8 | 110.7 | -93.1% | 29.6 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 993.1 | 53.5 | 256.1 | 1.99 | 37.9 | 257.1 | -74.2% | 19.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | pss_mib | 43.9 | 0.01 | 44.1 | 0.01 | 0.01 | 0.04 | +0.4% | 21.0 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2676 | 114.8 | -2469 | 103.2 | 109.1 | 2590 | -7.7% | 1.89 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 96.1 | 5.43 | 91.9 | 3.40 | 4.53 | 5.46 | -4.4% | 0.94 | BELOW FLOOR |
| P vs M | q1_p99_us | 706.1 | 296.5 | 449.5 | 71.7 | 215.7 | 65.3 | -36.3% | 1.19 | no difference |
| P vs M | q2_p50_us | 182.4 | 16.6 | 157.3 | 16.4 | 16.5 | 24.3 | -13.8% | 1.52 | no difference |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 215.8 | 16.6 | 203.7 | 29.8 | 24.1 | 37.1 | -5.6% | 0.50 | BELOW FLOOR |
| P vs M | q4_p50_us | 180.9 | 10.1 | 191.2 | 16.6 | 13.8 | 38.2 | +5.7% | 0.75 | BELOW FLOOR |
| P vs M | q5_p50_us | 136.1 | 20.7 | 124.6 | 18.6 | 19.7 | 90.3 | -8.4% | 0.58 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 99.3 | 11.5 | 79.7 | 52.8 | 38.2 | 14.1 | -19.8% | 0.51 | no difference |
| P vs M | c2_q1_p99_us | 1831 | 194.0 | 1369 | 342.4 | 278.3 | 205.3 | -25.3% | 1.66 | no difference |
| P vs M | c2_q2_p50_us | 182.4 | 31.3 | 153.8 | 69.9 | 54.1 | 32.3 | -15.7% | 0.53 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 150.9 | 36.2 | 190.0 | 20.3 | 29.4 | 181.1 | +25.9% | 1.33 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2280 | 283.6 | 1934 | 180.6 | 237.8 | 2433 | -15.2% | 1.45 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 247.7 | 21.5 | 297.1 | 22.6 | 22.0 | 106.4 | +20.0% | 2.24 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p99_us | 2361 | 1058 | 2181 | 347.1 | 787.4 | 578.4 | -7.6% | 0.23 | BELOW FLOOR |
| P vs M | write_p50_us | 12935 | 162.6 | 12908 | 441.2 | 332.5 | 862.0 | -0.2% | 0.08 | BELOW FLOOR |
| P vs M | write_p99_us | 18740 | 1287 | 18423 | 898.5 | 1110 | 6755 | -1.7% | 0.29 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | commits_per_s | 72.0 | 2.48 | 73.9 | 4.13 | 3.40 | 6.81 | +2.5% | 0.54 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 256.1 | 1.99 | 256.1 | 1.99 | 1.99 | 170.0 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.1 | 0.01 | 44.2 | 0.01 | 0.01 | 0.04 | +0.1% | 4.77 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2469 | 103.2 | -2471 | 129.9 | 117.3 | 2508 | +0.1% | 0.01 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 111.9 | 7.95 | 239.4 | 17.3 | 13.4 | 51.1 | +114.0% | 9.48 | **N better** |
| N vs H3 | q1_p99_us | 449.5 | 172.7 | 890.0 | 235.7 | 206.6 | 832.1 | +98.0% | 2.13 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 132.0 | 10.2 | 235.4 | 20.8 | 16.3 | 29.5 | +78.4% | 6.33 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 134.3 | 4.96 | 259.4 | 19.4 | 14.2 | 9.87 | +93.2% | 8.82 | **N better** |
| N vs H3 | q4_p50_us | 2176 | 57.9 | 1391 | 47.1 | 52.8 | 80.9 | -36.1% | 14.9 | **H3 better** |
| N vs H3 | q5_p50_us | 2372 | 29.2 | 3140 | 46.8 | 39.0 | 218.5 | +32.4% | 19.7 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 141.1 | 11.8 | 190.3 | 16.6 | 14.4 | 51.1 | +34.9% | 3.42 | BELOW FLOOR |
| N vs H3 | c2_q1_p99_us | 665.9 | 106.3 | 626.5 | 116.6 | 111.6 | 651.6 | -5.9% | 0.35 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 162.3 | 13.7 | 201.4 | 15.3 | 14.5 | 55.3 | +24.1% | 2.70 | BELOW FLOOR |
| N vs H3 | c4_q1_p50_us | 235.2 | 33.8 | 306.0 | 24.6 | 29.6 | 105.4 | +30.1% | 2.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 954.5 | 110.1 | 1169 | 147.1 | 130.0 | 832.8 | +22.4% | 1.65 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 274.1 | 31.1 | 312.2 | 39.7 | 35.7 | 197.5 | +13.9% | 1.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1495 | 333.2 | 1105 | 146.6 | 257.4 | 1056 | -26.1% | 1.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 452.3 | 21.7 | 1152 | 52.1 | 39.9 | 312.6 | +154.7% | 17.5 | **N better** |
| N vs H3 | write_p99_us | 1002 | 185.6 | 3775 | 302.3 | 250.8 | 613.2 | +276.8% | 11.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1942 | 96.1 | 737.1 | 57.9 | 79.3 | 297.4 | -62.1% | 15.2 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | view_state_bytes_per_key_read | 333.6 | 8.16 | 107.8 | 0.83 | 5.80 | 167.2 | -67.7% | 39.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 9.98 | 0.11 | 70.2 | 0.11 | 0.11 | 0.10 | +603.6% | 538.6 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 4785 | 19.0 | 16906 | 1014 | 717.4 | 12526 | +253.3% | 16.9 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 594.8 | 26.9 | 239.4 | 17.3 | 22.6 | 76.2 | -59.8% | 15.7 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3231 | 940.6 | 890.0 | 235.7 | 685.7 | 1091 | -72.5% | 3.41 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 446.8 | 15.1 | 235.4 | 20.8 | 18.1 | 118.4 | -47.3% | 11.7 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 249.4 | 24.0 | 259.4 | 19.4 | 21.8 | 91.0 | +4.0% | 0.46 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1319 | 32.6 | 1391 | 47.1 | 40.5 | 42.6 | +5.5% | 1.78 | no difference |
| P+ vs H3 | q5_p50_us | 3085 | 62.8 | 3140 | 46.8 | 55.4 | 463.2 | +1.8% | 1.00 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 647.2 | 49.3 | 190.3 | 16.6 | 36.8 | 59.6 | -70.6% | 12.4 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5170 | 540.4 | 626.5 | 116.6 | 390.9 | 695.4 | -87.9% | 11.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 526.1 | 21.0 | 201.4 | 15.3 | 18.4 | 63.8 | -61.7% | 17.7 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1796 | 104.6 | 306.0 | 24.6 | 76.0 | 24.8 | -83.0% | 19.6 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 8963 | 2295 | 1169 | 147.1 | 1626 | 820.3 | -87.0% | 4.79 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1578 | 127.6 | 312.2 | 39.7 | 94.5 | 359.1 | -80.2% | 13.4 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5763 | 1917 | 1105 | 146.6 | 1359 | 2052 | -80.8% | 3.43 | **H3 better** |
| P+ vs H3 | write_p50_us | 828.9 | 52.5 | 1152 | 52.1 | 52.3 | 313.7 | +39.0% | 6.18 | **P+ better** |
| P+ vs H3 | write_p99_us | 2572 | 160.8 | 3775 | 302.3 | 242.1 | 496.0 | +46.8% | 4.97 | **P+ better** |
| P+ vs H3 | commits_per_s | 1045 | 46.4 | 737.1 | 57.9 | 52.4 | 267.4 | -29.5% | 5.88 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | view_state_bytes_per_key_read | 993.1 | 53.5 | 107.8 | 0.83 | 37.8 | 227.4 | -89.1% | 23.4 | **H3 better** |
| P+ vs H3 | pss_mib | 43.9 | 0.01 | 70.2 | 0.11 | 0.08 | 0.05 | +59.8% | 335.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2676 | 114.8 | 16906 | 1014 | 721.8 | 12512 | -731.9% | 27.1 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 112.7 | 11.4 | 596.2 | 40.8 | 30.0 | 19.3 | +428.8% | 16.1 | **N better** |
| N vs P+ | q1_p99_us | 377.2 | 73.4 | 1515 | 505.7 | 361.3 | 724.7 | +301.6% | 3.15 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 145.2 | 15.3 | 464.0 | 13.9 | 14.6 | 26.2 | +219.6% | 21.8 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 145.2 | 18.4 | 242.2 | 19.7 | 19.1 | 99.6 | +66.8% | 5.09 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | 2157 | 62.4 | 1330 | 42.1 | 53.2 | 268.4 | -38.3% | 15.5 | **P+ better** |
| N vs P+ | q5_p50_us | 2414 | 109.4 | 2996 | 61.8 | 88.8 | 187.0 | +24.1% | 6.54 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 127.9 | 18.1 | 838.1 | 95.4 | 68.7 | 163.0 | +555.2% | 10.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 608.6 | 237.7 | 5473 | 2093 | 1489 | 1675 | +799.4% | 3.27 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 162.9 | 18.5 | 652.4 | 110.9 | 79.5 | 163.4 | +300.4% | 6.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 219.4 | 12.9 | 1849 | 131.8 | 93.6 | 436.0 | +743.1% | 17.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 840.8 | 164.4 | 11259 | 3225 | 2284 | 2135 | +1239.1% | 4.56 | **N better** |
| N vs P+ | c4_q2_p50_us | 235.4 | 22.7 | 1548 | 180.8 | 128.8 | 192.4 | +557.5% | 10.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1036 | 219.6 | 5813 | 2145 | 1524 | 339.6 | +461.2% | 3.13 | **N better** |
| N vs P+ | write_p50_us | 474.9 | 21.9 | 819.1 | 108.9 | 78.6 | 71.0 | +72.5% | 4.38 | **N better** |
| N vs P+ | write_p99_us | 934.9 | 219.1 | 2612 | 353.2 | 293.9 | 1640 | +179.4% | 5.71 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1908 | 92.6 | 1039 | 87.1 | 89.9 | 597.1 | -45.6% | 9.67 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 337.1 | 8.99 | 951.4 | 56.6 | 40.5 | 278.8 | +182.2% | 15.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 10.1 | 0.15 | 44.0 | 0.01 | 0.11 | 1.09 | +335.5% | 310.5 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4949 | 52.5 | -2542 | 59.7 | 56.2 | 1858 | -151.4% | 133.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 112.7 | 11.4 | 93.4 | 6.15 | 9.18 | 34.6 | -17.1% | 2.10 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q1_p99_us | 377.2 | 73.4 | 480.9 | 82.3 | 78.0 | 205.3 | +27.5% | 1.33 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 145.2 | 15.3 | 181.3 | 10.1 | 13.0 | 72.7 | +24.9% | 2.79 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 145.2 | 18.4 | 185.5 | 24.9 | 21.9 | 80.8 | +27.8% | 1.84 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q4_p50_us | 2157 | 62.4 | 196.9 | 22.2 | 46.8 | 310.6 | -90.9% | 41.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q5_p50_us | 2414 | 109.4 | 117.3 | 8.81 | 77.6 | 105.8 | -95.1% | 29.6 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 127.9 | 18.1 | 90.7 | 24.3 | 21.5 | 134.7 | -29.1% | 1.74 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 608.6 | 237.7 | 1331 | 480.2 | 378.9 | 445.5 | +118.7% | 1.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 162.9 | 18.5 | 170.9 | 32.1 | 26.2 | 135.8 | +4.9% | 0.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 219.4 | 12.9 | 193.9 | 38.6 | 28.8 | 217.8 | -11.6% | 0.89 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 840.8 | 164.4 | 2449 | 347.5 | 271.9 | 926.1 | +191.3% | 5.92 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 235.4 | 22.7 | 261.3 | 28.3 | 25.6 | 192.7 | +11.0% | 1.01 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 1036 | 219.6 | 2346 | 291.8 | 258.3 | 140.9 | +126.5% | 5.07 | **N better** |
| N vs P | write_p50_us | 474.9 | 21.9 | 12959 | 244.8 | 173.8 | 2058 | +2628.8% | 71.8 | **N better** |
| N vs P | write_p99_us | 934.9 | 219.1 | 18805 | 1216 | 873.8 | 2328 | +1911.5% | 20.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1908 | 92.6 | 73.6 | 0.81 | 65.5 | 597.2 | -96.1% | 28.0 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | view_state_bytes_per_key_read | 337.1 | 8.99 | 257.8 | 3.33 | 6.78 | 215.3 | -23.5% | 11.7 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 10.1 | 0.15 | 44.2 | 0.02 | 0.11 | 1.09 | +337.1% | 310.5 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 4949 | 52.5 | -2396 | 94.9 | 76.7 | 1738 | -148.4% | 95.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 596.2 | 40.8 | 93.4 | 6.15 | 29.2 | 38.5 | -84.3% | 17.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p99_us | 1515 | 505.7 | 480.9 | 82.3 | 362.3 | 708.4 | -68.3% | 2.85 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 464.0 | 13.9 | 181.3 | 10.1 | 12.2 | 73.1 | -60.9% | 23.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 242.2 | 19.7 | 185.5 | 24.9 | 22.5 | 72.1 | -23.4% | 2.52 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1330 | 42.1 | 196.9 | 22.2 | 33.7 | 238.6 | -85.2% | 33.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q5_p50_us | 2996 | 61.8 | 117.3 | 8.81 | 44.1 | 158.7 | -96.1% | 65.2 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 838.1 | 95.4 | 90.7 | 24.3 | 69.6 | 113.0 | -89.2% | 10.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5473 | 2093 | 1331 | 480.2 | 1518 | 1618 | -75.7% | 2.73 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 652.4 | 110.9 | 170.9 | 32.1 | 81.6 | 103.5 | -73.8% | 5.90 | **P better** |
| P+ vs P | c4_q1_p50_us | 1849 | 131.8 | 193.9 | 38.6 | 97.1 | 426.9 | -89.5% | 17.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 11259 | 3225 | 2449 | 347.5 | 2294 | 2316 | -78.2% | 3.84 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1548 | 180.8 | 261.3 | 28.3 | 129.4 | 229.1 | -83.1% | 9.94 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 5813 | 2145 | 2346 | 291.8 | 1530 | 311.0 | -59.6% | 2.27 | no difference |
| P+ vs P | write_p50_us | 819.1 | 108.9 | 12959 | 244.8 | 189.5 | 2059 | +1482.1% | 64.1 | **P+ better** |
| P+ vs P | write_p99_us | 2612 | 353.2 | 18805 | 1216 | 895.5 | 1653 | +620.0% | 18.1 | **P+ better** |
| P+ vs P | commits_per_s | 1039 | 87.1 | 73.6 | 0.81 | 61.6 | 39.4 | -92.9% | 15.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | view_state_bytes_per_key_read | 951.4 | 56.6 | 257.8 | 3.33 | 40.1 | 265.4 | -72.9% | 17.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | pss_mib | 44.0 | 0.01 | 44.2 | 0.02 | 0.01 | 0.03 | +0.4% | 12.7 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2542 | 59.7 | -2396 | 94.9 | 79.3 | 2435 | -5.8% | 1.85 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 93.4 | 6.15 | 101.1 | 15.8 | 12.0 | 34.4 | +8.2% | 0.64 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q1_p99_us | 480.9 | 82.3 | 484.4 | 57.8 | 71.2 | 417.4 | +0.7% | 0.05 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 181.3 | 10.1 | 178.0 | 18.5 | 14.9 | 70.6 | -1.8% | 0.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 185.5 | 24.9 | 225.4 | 59.0 | 45.3 | 103.0 | +21.5% | 0.88 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 196.9 | 22.2 | 210.4 | 37.0 | 30.5 | 213.7 | +6.8% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q5_p50_us | 117.3 | 8.81 | 112.8 | 8.23 | 8.53 | 75.5 | -3.9% | 0.53 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 90.7 | 24.3 | 67.7 | 36.7 | 31.1 | 46.6 | -25.3% | 0.74 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1331 | 480.2 | 1206 | 207.5 | 369.9 | 634.3 | -9.4% | 0.34 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 170.9 | 32.1 | 150.7 | 64.4 | 50.9 | 35.1 | -11.8% | 0.40 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 193.9 | 38.6 | 151.1 | 30.2 | 34.7 | 140.9 | -22.1% | 1.23 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2449 | 347.5 | 2124 | 260.6 | 307.2 | 1036 | -13.3% | 1.06 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 261.3 | 28.3 | 237.6 | 28.4 | 28.3 | 167.4 | -9.1% | 0.84 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 2346 | 291.8 | 2077 | 211.0 | 254.6 | 190.9 | -11.4% | 1.05 | no difference |
| P vs M | write_p50_us | 12959 | 244.8 | 13381 | 673.5 | 506.7 | 2085 | +3.3% | 0.83 | BELOW FLOOR |
| P vs M | write_p99_us | 18805 | 1216 | 19423 | 2652 | 2063 | 1809 | +3.3% | 0.30 | BELOW FLOOR |
| P vs M | commits_per_s | 73.6 | 0.81 | 70.0 | 3.51 | 2.54 | 29.2 | -5.0% | 1.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | view_state_bytes_per_key_read | 257.8 | 3.33 | 257.8 | 3.33 | 3.33 | 197.6 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.2 | 0.02 | 44.3 | 0.03 | 0.02 | 0.04 | +0.2% | 4.50 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2396 | 94.9 | -2380 | 52.6 | 76.7 | 2363 | -0.6% | 0.20 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 112.7 | 11.4 | 219.4 | 14.2 | 12.9 | 10.3 | +94.6% | 8.28 | **N better** |
| N vs H3 | q1_p99_us | 377.2 | 73.4 | 616.8 | 122.8 | 101.2 | 208.0 | +63.5% | 2.37 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 145.2 | 15.3 | 248.5 | 28.4 | 22.8 | 72.9 | +71.1% | 4.52 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 145.2 | 18.4 | 302.3 | 27.1 | 23.2 | 85.2 | +108.2% | 6.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | 2157 | 62.4 | 1370 | 39.8 | 52.3 | 257.6 | -36.5% | 15.1 | **H3 better** |
| N vs H3 | q5_p50_us | 2414 | 109.4 | 3212 | 132.6 | 121.6 | 170.9 | +33.0% | 6.56 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 127.9 | 18.1 | 212.4 | 31.0 | 25.4 | 131.0 | +66.0% | 3.33 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 608.6 | 237.7 | 996.1 | 318.1 | 280.8 | 520.5 | +63.7% | 1.38 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 162.9 | 18.5 | 244.5 | 39.2 | 30.6 | 132.6 | +50.1% | 2.66 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 219.4 | 12.9 | 356.1 | 48.8 | 35.7 | 263.8 | +62.3% | 3.83 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 840.8 | 164.4 | 1598 | 114.4 | 141.6 | 513.4 | +90.1% | 5.35 | **N better** |
| N vs H3 | c4_q2_p50_us | 235.4 | 22.7 | 397.0 | 31.6 | 27.5 | 204.3 | +68.6% | 5.87 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p99_us | 1036 | 219.6 | 1907 | 336.6 | 284.2 | 574.3 | +84.1% | 3.06 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 474.9 | 21.9 | 1245 | 90.5 | 65.8 | 255.2 | +162.2% | 11.7 | **N better** |
| N vs H3 | write_p99_us | 934.9 | 219.1 | 4570 | 1220 | 876.7 | 1641 | +388.9% | 4.15 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1908 | 92.6 | 604.7 | 69.8 | 82.0 | 597.3 | -68.3% | 15.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 337.1 | 8.99 | 109.6 | 1.35 | 6.43 | 163.9 | -67.5% | 35.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 10.1 | 0.15 | 70.8 | 0.23 | 0.19 | 1.16 | +600.6% | 312.7 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 4949 | 52.5 | 17482 | 606.8 | 430.7 | 8152 | +253.2% | 29.1 | **N better** |
| P+ vs H3 | q1_p50_us | 596.2 | 40.8 | 219.4 | 14.2 | 30.6 | 19.8 | -63.2% | 12.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1515 | 505.7 | 616.8 | 122.8 | 367.9 | 709.2 | -59.3% | 2.44 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 464.0 | 13.9 | 248.5 | 28.4 | 22.4 | 73.3 | -46.4% | 9.62 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 242.2 | 19.7 | 302.3 | 27.1 | 23.7 | 77.0 | +24.8% | 2.54 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1330 | 42.1 | 1370 | 39.8 | 40.9 | 163.8 | +2.9% | 0.95 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 2996 | 61.8 | 3212 | 132.6 | 103.5 | 207.9 | +7.2% | 2.09 | no difference |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 838.1 | 95.4 | 212.4 | 31.0 | 70.9 | 108.5 | -74.7% | 8.82 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5473 | 2093 | 996.1 | 318.1 | 1497 | 1640 | -81.8% | 2.99 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 652.4 | 110.9 | 244.5 | 39.2 | 83.2 | 99.2 | -62.5% | 4.90 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1849 | 131.8 | 356.1 | 48.8 | 99.4 | 452.1 | -80.7% | 15.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 11259 | 3225 | 1598 | 114.4 | 2282 | 2184 | -85.8% | 4.23 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1548 | 180.8 | 397.0 | 31.6 | 129.8 | 238.9 | -74.4% | 8.87 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 5813 | 2145 | 1907 | 336.6 | 1535 | 637.7 | -67.2% | 2.54 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 819.1 | 108.9 | 1245 | 90.5 | 100.2 | 264.9 | +52.0% | 4.25 | **P+ better** |
| P+ vs H3 | write_p99_us | 2612 | 353.2 | 4570 | 1220 | 898.3 | 78.4 | +75.0% | 2.18 | no difference |
| P+ vs H3 | commits_per_s | 1039 | 87.1 | 604.7 | 69.8 | 78.9 | 41.3 | -41.8% | 5.50 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 951.4 | 56.6 | 109.6 | 1.35 | 40.0 | 225.6 | -88.5% | 21.0 | **H3 better** |
| P+ vs H3 | pss_mib | 44.0 | 0.01 | 70.8 | 0.23 | 0.16 | 0.42 | +60.9% | 166.8 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2542 | 59.7 | 17482 | 606.8 | 431.2 | 8329 | -787.6% | 46.4 | REFUSED (warm-up MAD above 15% of the median on P+) |

## `single`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 110.7 | 7.61 | 692.7 | 33.3 | 24.2 | 9.71 | +525.8% | 24.1 | **N better** |
| N vs P+ | q1_p99_us | 373.8 | 75.6 | 1937 | 735.7 | 522.9 | 2952 | +418.4% | 2.99 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 129.9 | 8.90 | 544.7 | 27.0 | 20.1 | 32.3 | +319.2% | 20.7 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 136.1 | 18.0 | 294.8 | 25.4 | 22.1 | 61.6 | +116.7% | 7.20 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | 2143 | 62.3 | 1401 | 103.8 | 85.6 | 134.5 | -34.6% | 8.67 | **P+ better** |
| N vs P+ | q5_p50_us | 2464 | 72.6 | 3249 | 143.3 | 113.6 | 230.1 | +31.9% | 6.91 | **N better** |
| N vs P+ | q6_p50_us | 3439 | 160.2 | 3592 | 495.4 | 368.2 | 136.5 | +4.4% | 0.42 | no difference |
| N vs P+ | c2_q1_p50_us | 56.1 | 8.53 | 729.9 | 63.1 | 45.0 | 38.5 | +1201.2% | 15.0 | **N better** |
| N vs P+ | c2_q1_p99_us | 276.4 | 65.3 | 6738 | 1023 | 725.0 | 1021 | +2338.1% | 8.91 | **N better** |
| N vs P+ | c2_q2_p50_us | 75.8 | 20.4 | 580.8 | 57.7 | 43.3 | 40.8 | +665.8% | 11.7 | **N better** |
| N vs P+ | c4_q1_p50_us | 105.0 | 12.3 | 1821 | 74.8 | 53.6 | 25.2 | +1635.1% | 32.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 586.5 | 127.8 | 11487 | 5301 | 3749 | 2123 | +1858.5% | 2.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 119.4 | 12.0 | 1556 | 83.1 | 59.4 | 282.9 | +1203.8% | 24.2 | **N better** |
| N vs P+ | c4_q2_p99_us | 796.4 | 179.4 | 13612 | 5328 | 3770 | 2397 | +1609.2% | 3.40 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 469.8 | 33.8 | 929.5 | 71.8 | 56.1 | 143.8 | +97.9% | 8.20 | **N better** |
| N vs P+ | write_p99_us | 782.5 | 125.6 | 2739 | 164.0 | 146.1 | 803.3 | +250.1% | 13.4 | **N better** |
| N vs P+ | commits_per_s | 1981 | 151.7 | 911.7 | 77.0 | 120.3 | 174.1 | -54.0% | 8.88 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 329.9 | 4.64 | 945.3 | 62.1 | 44.0 | 280.2 | +186.6% | 14.0 | **N better** |
| N vs P+ | pss_mib | 9.38 | 0.27 | 44.2 | 0.03 | 0.19 | 0.29 | +371.6% | 180.5 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4131 | 106.6 | -2469 | 161.5 | 136.8 | 1689 | -159.8% | 48.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 110.7 | 7.61 | 92.2 | 8.02 | 7.82 | 34.3 | -16.7% | 2.37 | BELOW FLOOR |
| N vs P | q1_p99_us | 373.8 | 75.6 | 541.4 | 108.8 | 93.7 | 155.8 | +44.9% | 1.79 | no difference |
| N vs P | q2_p50_us | 129.9 | 8.90 | 196.2 | 29.2 | 21.6 | 43.9 | +51.0% | 3.07 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 136.1 | 18.0 | 208.8 | 20.6 | 19.4 | 61.6 | +53.5% | 3.76 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q4_p50_us | 2143 | 62.3 | 216.4 | 30.0 | 48.9 | 141.1 | -89.9% | 39.4 | **P better** |
| N vs P | q5_p50_us | 2464 | 72.6 | 117.5 | 7.75 | 51.6 | 231.8 | -95.2% | 45.5 | **P better** |
| N vs P | q6_p50_us | 3439 | 160.2 | 473.3 | 30.5 | 115.3 | 132.3 | -86.2% | 25.7 | **P better** |
| N vs P | c2_q1_p50_us | 56.1 | 8.53 | 106.4 | 31.2 | 22.8 | 47.6 | +89.7% | 2.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 276.4 | 65.3 | 2279 | 693.3 | 492.4 | 135.7 | +724.5% | 4.07 | **N better** |
| N vs P | c2_q2_p50_us | 75.8 | 20.4 | 212.1 | 47.7 | 36.7 | 33.7 | +179.7% | 3.71 | **N better** |
| N vs P | c4_q1_p50_us | 105.0 | 12.3 | 179.1 | 35.1 | 26.3 | 11.8 | +70.6% | 2.82 | no difference |
| N vs P | c4_q1_p99_us | 586.5 | 127.8 | 2437 | 443.3 | 326.2 | 689.0 | +315.4% | 5.67 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 119.4 | 12.0 | 298.9 | 30.9 | 23.4 | 7.59 | +150.4% | 7.66 | **N better** |
| N vs P | c4_q2_p99_us | 796.4 | 179.4 | 2533 | 661.0 | 484.3 | 2123 | +218.1% | 3.59 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 469.8 | 33.8 | 12783 | 176.3 | 126.9 | 355.3 | +2621.2% | 97.0 | **N better** |
| N vs P | write_p99_us | 782.5 | 125.6 | 21563 | 2803 | 1984 | 554.7 | +2655.7% | 10.5 | **N better** |
| N vs P | commits_per_s | 1981 | 151.7 | 72.6 | 2.77 | 107.3 | 170.5 | -96.3% | 17.8 | **N better** |
| N vs P | view_state_bytes_per_key_read | 329.9 | 4.64 | 252.2 | 5.23 | 4.94 | 151.5 | -23.6% | 15.7 | BELOW FLOOR |
| N vs P | pss_mib | 9.38 | 0.27 | 44.2 | 0.01 | 0.19 | 0.29 | +371.8% | 181.4 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 4131 | 106.6 | -2409 | 100.4 | 103.6 | 1625 | -158.3% | 63.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 692.7 | 33.3 | 92.2 | 8.02 | 24.2 | 34.8 | -86.7% | 24.8 | **P better** |
| P+ vs P | q1_p99_us | 1937 | 735.7 | 541.4 | 108.8 | 525.9 | 2956 | -72.1% | 2.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 544.7 | 27.0 | 196.2 | 29.2 | 28.1 | 45.3 | -64.0% | 12.4 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 294.8 | 25.4 | 208.8 | 20.6 | 23.2 | 51.6 | -29.2% | 3.71 | **P better** |
| P+ vs P | q4_p50_us | 1401 | 103.8 | 216.4 | 30.0 | 76.4 | 54.3 | -84.5% | 15.5 | **P better** |
| P+ vs P | q5_p50_us | 3249 | 143.3 | 117.5 | 7.75 | 101.5 | 33.8 | -96.4% | 30.8 | **P better** |
| P+ vs P | q6_p50_us | 3592 | 495.4 | 473.3 | 30.5 | 351.0 | 74.0 | -86.8% | 8.89 | **P better** |
| P+ vs P | c2_q1_p50_us | 729.9 | 63.1 | 106.4 | 31.2 | 49.7 | 56.5 | -85.4% | 12.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 6738 | 1023 | 2279 | 693.3 | 873.9 | 1019 | -66.2% | 5.10 | **P better** |
| P+ vs P | c2_q2_p50_us | 580.8 | 57.7 | 212.1 | 47.7 | 53.0 | 51.7 | -63.5% | 6.96 | **P better** |
| P+ vs P | c4_q1_p50_us | 1821 | 74.8 | 179.1 | 35.1 | 58.4 | 27.4 | -90.2% | 28.1 | **P better** |
| P+ vs P | c4_q1_p99_us | 11487 | 5301 | 2437 | 443.3 | 3761 | 2175 | -78.8% | 2.41 | no difference |
| P+ vs P | c4_q2_p50_us | 1556 | 83.1 | 298.9 | 30.9 | 62.7 | 283.0 | -80.8% | 20.1 | **P better** |
| P+ vs P | c4_q2_p99_us | 13612 | 5328 | 2533 | 661.0 | 3796 | 3129 | -81.4% | 2.92 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 929.5 | 71.8 | 12783 | 176.3 | 134.6 | 382.6 | +1275.2% | 88.1 | **P+ better** |
| P+ vs P | write_p99_us | 2739 | 164.0 | 21563 | 2803 | 1986 | 971.2 | +687.2% | 9.48 | **P+ better** |
| P+ vs P | commits_per_s | 911.7 | 77.0 | 72.6 | 2.77 | 54.5 | 35.4 | -92.0% | 15.4 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 945.3 | 62.1 | 252.2 | 5.23 | 44.1 | 262.2 | -73.3% | 15.7 | **P better** |
| P+ vs P | pss_mib | 44.2 | 0.03 | 44.2 | 0.01 | 0.02 | 0.01 | +0.0% | 0.80 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2469 | 161.5 | -2409 | 100.4 | 134.5 | 2317 | -2.4% | 0.45 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 92.2 | 8.02 | 96.5 | 4.83 | 6.62 | 35.1 | +4.7% | 0.66 | BELOW FLOOR |
| P vs M | q1_p99_us | 541.4 | 108.8 | 520.0 | 157.5 | 135.4 | 161.1 | -4.0% | 0.16 | BELOW FLOOR |
| P vs M | q2_p50_us | 196.2 | 29.2 | 185.8 | 18.6 | 24.5 | 38.4 | -5.3% | 0.42 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 208.8 | 20.6 | 208.4 | 35.4 | 29.0 | 40.3 | -0.2% | 0.01 | BELOW FLOOR |
| P vs M | q4_p50_us | 216.4 | 30.0 | 206.2 | 16.8 | 24.3 | 115.3 | -4.7% | 0.42 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q5_p50_us | 117.5 | 7.75 | 109.4 | 3.20 | 5.93 | 42.9 | -6.9% | 1.36 | BELOW FLOOR |
| P vs M | q6_p50_us | 473.3 | 30.5 | 483.5 | 97.2 | 72.0 | 47.1 | +2.2% | 0.14 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 106.4 | 31.2 | 92.6 | 22.9 | 27.4 | 45.0 | -13.0% | 0.51 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 2279 | 693.3 | 1292 | 227.2 | 515.9 | 515.2 | -43.3% | 1.91 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 212.1 | 47.7 | 160.7 | 30.1 | 39.9 | 45.5 | -24.2% | 1.29 | no difference |
| P vs M | c4_q1_p50_us | 179.1 | 35.1 | 87.0 | 33.4 | 34.3 | 52.2 | -51.4% | 2.69 | no difference |
| P vs M | c4_q1_p99_us | 2437 | 443.3 | 2178 | 327.8 | 389.8 | 595.9 | -10.6% | 0.66 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 298.9 | 30.9 | 203.4 | 50.6 | 41.9 | 13.0 | -32.0% | 2.28 | no difference |
| P vs M | c4_q2_p99_us | 2533 | 661.0 | 2041 | 518.3 | 594.0 | 2264 | -19.4% | 0.83 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 12783 | 176.3 | 12921 | 151.1 | 164.2 | 774.4 | +1.1% | 0.84 | BELOW FLOOR |
| P vs M | write_p99_us | 21563 | 2803 | 20689 | 3628 | 3242 | 5180 | -4.1% | 0.27 | BELOW FLOOR |
| P vs M | commits_per_s | 72.6 | 2.77 | 73.1 | 2.01 | 2.42 | 14.3 | +0.7% | 0.22 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 252.2 | 5.23 | 254.4 | 2.81 | 4.20 | 114.8 | +0.9% | 0.52 | BELOW FLOOR |
| P vs M | pss_mib | 44.2 | 0.01 | 44.1 | 0.02 | 0.02 | 0.03 | -0.4% | 9.47 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2409 | 100.4 | -2464 | 100.5 | 100.5 | 2355 | +2.3% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 110.7 | 7.61 | 231.5 | 17.5 | 13.5 | 8.97 | +109.2% | 8.94 | **N better** |
| N vs H3 | q1_p99_us | 373.8 | 75.6 | 732.5 | 92.6 | 84.5 | 39.2 | +96.0% | 4.24 | **N better** |
| N vs H3 | q2_p50_us | 129.9 | 8.90 | 242.6 | 12.1 | 10.6 | 21.6 | +86.7% | 10.6 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 136.1 | 18.0 | 252.5 | 21.0 | 19.6 | 117.7 | +85.6% | 5.95 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | 2143 | 62.3 | 1346 | 82.4 | 73.0 | 135.4 | -37.2% | 10.9 | **H3 better** |
| N vs H3 | q5_p50_us | 2464 | 72.6 | 3138 | 111.0 | 93.8 | 385.0 | +27.4% | 7.19 | **N better** |
| N vs H3 | q6_p50_us | 3439 | 160.2 | 3515 | 80.5 | 126.8 | 185.3 | +2.2% | 0.60 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 56.1 | 8.53 | 189.1 | 18.3 | 14.3 | 33.2 | +237.1% | 9.33 | **N better** |
| N vs H3 | c2_q1_p99_us | 276.4 | 65.3 | 857.3 | 157.9 | 120.8 | 134.8 | +210.2% | 4.81 | **N better** |
| N vs H3 | c2_q2_p50_us | 75.8 | 20.4 | 211.5 | 17.2 | 18.9 | 44.5 | +178.9% | 7.18 | **N better** |
| N vs H3 | c4_q1_p50_us | 105.0 | 12.3 | 345.0 | 29.5 | 22.6 | 149.1 | +228.7% | 10.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 586.5 | 127.8 | 1284 | 249.0 | 197.9 | 1329 | +118.9% | 3.52 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 119.4 | 12.0 | 366.1 | 45.3 | 33.1 | 151.6 | +206.6% | 7.44 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 796.4 | 179.4 | 1229 | 191.4 | 185.5 | 2949 | +54.3% | 2.33 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 469.8 | 33.8 | 1196 | 79.6 | 61.2 | 43.7 | +154.6% | 11.9 | **N better** |
| N vs H3 | write_p99_us | 782.5 | 125.6 | 3802 | 573.2 | 414.9 | 930.5 | +385.9% | 7.28 | **N better** |
| N vs H3 | commits_per_s | 1981 | 151.7 | 624.8 | 96.2 | 127.0 | 170.9 | -68.5% | 10.7 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 329.9 | 4.64 | 112.0 | 0.83 | 3.33 | 128.1 | -66.1% | 65.4 | **H3 better** |
| N vs H3 | pss_mib | 9.38 | 0.27 | 70.0 | 0.30 | 0.29 | 0.29 | +647.0% | 211.0 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 4131 | 106.6 | 16720 | 446.1 | 324.3 | 10676 | +304.7% | 38.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 692.7 | 33.3 | 231.5 | 17.5 | 26.6 | 10.7 | -66.6% | 17.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1937 | 735.7 | 732.5 | 92.6 | 524.3 | 2952 | -62.2% | 2.30 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 544.7 | 27.0 | 242.6 | 12.1 | 20.9 | 24.3 | -55.5% | 14.5 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 294.8 | 25.4 | 252.5 | 21.0 | 23.3 | 112.8 | -14.3% | 1.81 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1401 | 103.8 | 1346 | 82.4 | 93.7 | 36.7 | -3.9% | 0.59 | no difference |
| P+ vs H3 | q5_p50_us | 3249 | 143.3 | 3138 | 111.0 | 128.2 | 309.2 | -3.4% | 0.86 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 3592 | 495.4 | 3515 | 80.5 | 354.9 | 149.3 | -2.1% | 0.22 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 729.9 | 63.1 | 189.1 | 18.3 | 46.4 | 45.0 | -74.1% | 11.7 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 6738 | 1023 | 857.3 | 157.9 | 732.1 | 1019 | -87.3% | 8.03 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 580.8 | 57.7 | 211.5 | 17.2 | 42.6 | 59.3 | -63.6% | 8.67 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1821 | 74.8 | 345.0 | 29.5 | 56.9 | 151.1 | -81.1% | 26.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 11487 | 5301 | 1284 | 249.0 | 3753 | 2454 | -88.8% | 2.72 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1556 | 83.1 | 366.1 | 45.3 | 66.9 | 321.0 | -76.5% | 17.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 13612 | 5328 | 1229 | 191.4 | 3770 | 3738 | -91.0% | 3.28 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 929.5 | 71.8 | 1196 | 79.6 | 75.8 | 148.3 | +28.6% | 3.51 | **P+ better** |
| P+ vs H3 | write_p99_us | 2739 | 164.0 | 3802 | 573.2 | 421.6 | 1225 | +38.8% | 2.52 | BELOW FLOOR |
| P+ vs H3 | commits_per_s | 911.7 | 77.0 | 624.8 | 96.2 | 87.1 | 37.1 | -31.5% | 3.29 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 945.3 | 62.1 | 112.0 | 0.83 | 43.9 | 249.4 | -88.2% | 19.0 | **H3 better** |
| P+ vs H3 | pss_mib | 44.2 | 0.03 | 70.0 | 0.30 | 0.22 | 0.06 | +58.4% | 120.1 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2469 | 161.5 | 16720 | 446.1 | 335.5 | 10803 | -777.1% | 57.2 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 105.0 | 7.65 | 609.0 | 58.0 | 41.4 | 118.9 | +480.1% | 12.2 | **N better** |
| N vs P+ | q1_p99_us | 327.3 | 63.0 | 1430 | 294.9 | 213.2 | 1098 | +336.8% | 5.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 126.7 | 14.0 | 454.5 | 58.5 | 42.5 | 43.2 | +258.8% | 7.71 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 132.7 | 15.2 | 273.5 | 30.1 | 23.8 | 58.3 | +106.2% | 5.91 | **N better** |
| N vs P+ | q4_p50_us | 2105 | 19.3 | 1376 | 139.0 | 99.3 | 39.9 | -34.6% | 7.34 | **P+ better** |
| N vs P+ | q5_p50_us | 2386 | 56.9 | 3142 | 152.7 | 115.2 | 73.2 | +31.7% | 6.57 | **N better** |
| N vs P+ | q6_p50_us | 3491 | 184.7 | 3262 | 177.7 | 181.3 | 579.6 | -6.6% | 1.26 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 64.5 | 10.9 | 738.9 | 130.2 | 92.4 | 211.8 | +1045.7% | 7.30 | **N better** |
| N vs P+ | c2_q1_p99_us | 312.0 | 70.2 | 5679 | 1501 | 1062 | 341.1 | +1720.3% | 5.05 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 81.2 | 13.4 | 570.7 | 76.9 | 55.2 | 307.9 | +603.0% | 8.87 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 113.7 | 13.0 | 1734 | 185.1 | 131.2 | 198.7 | +1425.2% | 12.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 624.6 | 169.9 | 10643 | 2206 | 1565 | 2106459 | +1604.0% | 6.40 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 134.7 | 22.6 | 1594 | 246.3 | 174.9 | 43.8 | +1083.3% | 8.34 | **N better** |
| N vs P+ | c4_q2_p99_us | 767.3 | 169.1 | 8035 | 2157 | 1530 | 13373 | +947.1% | 4.75 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 456.3 | 27.7 | 838.7 | 43.9 | 36.7 | 67.6 | +83.8% | 10.4 | **N better** |
| N vs P+ | write_p99_us | 755.2 | 33.7 | 2568 | 119.2 | 87.6 | 891.2 | +240.0% | 20.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 2107 | 78.2 | 1058 | 75.7 | 77.0 | 115.3 | -49.8% | 13.6 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 329.1 | 2.50 | 931.5 | 23.2 | 16.5 | 344.7 | +183.1% | 36.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 9.86 | 0.21 | 44.0 | 0.03 | 0.15 | 1.20 | +346.0% | 225.9 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4812 | 209.4 | -2566 | 214.2 | 211.8 | 1594 | -153.3% | 34.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 105.0 | 7.65 | 90.4 | 2.12 | 5.61 | 14.7 | -13.9% | 2.61 | BELOW FLOOR |
| N vs P | q1_p99_us | 327.3 | 63.0 | 453.2 | 72.0 | 67.6 | 266.0 | +38.5% | 1.86 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 126.7 | 14.0 | 185.1 | 21.2 | 17.9 | 8.45 | +46.1% | 3.26 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 132.7 | 15.2 | 195.8 | 9.67 | 12.7 | 107.2 | +47.6% | 4.96 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q4_p50_us | 2105 | 19.3 | 181.6 | 16.3 | 17.9 | 10.1 | -91.4% | 107.6 | **P better** |
| N vs P | q5_p50_us | 2386 | 56.9 | 121.2 | 12.9 | 41.2 | 65.0 | -94.9% | 54.9 | **P better** |
| N vs P | q6_p50_us | 3491 | 184.7 | 489.4 | 45.9 | 134.6 | 572.7 | -86.0% | 22.3 | **P better** |
| N vs P | c2_q1_p50_us | 64.5 | 10.9 | 49.5 | 24.7 | 19.1 | 55.1 | -23.3% | 0.79 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 312.0 | 70.2 | 1271 | 313.0 | 226.8 | 306.4 | +307.5% | 4.23 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 81.2 | 13.4 | 110.9 | 27.4 | 21.5 | 66.4 | +36.6% | 1.38 | BELOW FLOOR |
| N vs P | c4_q1_p50_us | 113.7 | 13.0 | 104.8 | 12.6 | 12.8 | 19.2 | -7.8% | 0.69 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 624.6 | 169.9 | 2225 | 244.5 | 210.5 | 868.9 | +256.2% | 7.60 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p50_us | 134.7 | 22.6 | 212.1 | 36.2 | 30.2 | 60.8 | +57.5% | 2.57 | no difference |
| N vs P | c4_q2_p99_us | 767.3 | 169.1 | 2360 | 531.1 | 394.1 | 766.1 | +207.5% | 4.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 456.3 | 27.7 | 12632 | 307.3 | 218.1 | 304.3 | +2668.0% | 55.8 | **N better** |
| N vs P | write_p99_us | 755.2 | 33.7 | 18289 | 1053 | 744.7 | 661.1 | +2321.7% | 23.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 2107 | 78.2 | 76.0 | 2.22 | 55.3 | 26.7 | -96.4% | 36.7 | **N better** |
| N vs P | view_state_bytes_per_key_read | 329.1 | 2.50 | 263.6 | 2.72 | 2.61 | 212.1 | -19.9% | 25.1 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 9.86 | 0.21 | 44.1 | 0.03 | 0.15 | 1.21 | +346.7% | 227.6 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 4812 | 209.4 | -2629 | 135.6 | 176.4 | 2002 | -154.6% | 42.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 609.0 | 58.0 | 90.4 | 2.12 | 41.1 | 119.8 | -85.2% | 12.6 | **P better** |
| P+ vs P | q1_p99_us | 1430 | 294.9 | 453.2 | 72.0 | 214.7 | 1066 | -68.3% | 4.55 | BELOW FLOOR |
| P+ vs P | q2_p50_us | 454.5 | 58.5 | 185.1 | 21.2 | 44.0 | 43.5 | -59.3% | 6.13 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 273.5 | 30.1 | 195.8 | 9.67 | 22.4 | 121.5 | -28.4% | 3.48 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 1376 | 139.0 | 181.6 | 16.3 | 99.0 | 39.5 | -86.8% | 12.1 | **P better** |
| P+ vs P | q5_p50_us | 3142 | 152.7 | 121.2 | 12.9 | 108.3 | 43.5 | -96.1% | 27.9 | **P better** |
| P+ vs P | q6_p50_us | 3262 | 177.7 | 489.4 | 45.9 | 129.8 | 91.2 | -85.0% | 21.4 | **P better** |
| P+ vs P | c2_q1_p50_us | 738.9 | 130.2 | 49.5 | 24.7 | 93.7 | 214.6 | -93.3% | 7.36 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5679 | 1501 | 1271 | 313.0 | 1084 | 181.8 | -77.6% | 4.07 | **P better** |
| P+ vs P | c2_q2_p50_us | 570.7 | 76.9 | 110.9 | 27.4 | 57.7 | 312.3 | -80.6% | 7.97 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p50_us | 1734 | 185.1 | 104.8 | 12.6 | 131.2 | 199.1 | -94.0% | 12.4 | **P better** |
| P+ vs P | c4_q1_p99_us | 10643 | 2206 | 2225 | 244.5 | 1570 | 2106459 | -79.1% | 5.36 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1594 | 246.3 | 212.1 | 36.2 | 176.0 | 42.8 | -86.7% | 7.85 | **P better** |
| P+ vs P | c4_q2_p99_us | 8035 | 2157 | 2360 | 531.1 | 1571 | 13352 | -70.6% | 3.61 | BELOW FLOOR |
| P+ vs P | write_p50_us | 838.7 | 43.9 | 12632 | 307.3 | 219.5 | 311.7 | +1406.1% | 53.7 | **P+ better** |
| P+ vs P | write_p99_us | 2568 | 119.2 | 18289 | 1053 | 749.0 | 599.4 | +612.2% | 21.0 | **P+ better** |
| P+ vs P | commits_per_s | 1058 | 75.7 | 76.0 | 2.22 | 53.6 | 112.5 | -92.8% | 18.3 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 931.5 | 23.2 | 263.6 | 2.72 | 16.6 | 335.0 | -71.7% | 40.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | pss_mib | 44.0 | 0.03 | 44.1 | 0.03 | 0.03 | 0.15 | +0.2% | 2.34 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2566 | 214.2 | -2629 | 135.6 | 179.2 | 2559 | +2.4% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 90.4 | 2.12 | 91.7 | 5.36 | 4.08 | 37.6 | +1.5% | 0.33 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q1_p99_us | 453.2 | 72.0 | 406.9 | 24.8 | 53.8 | 54.7 | -10.2% | 0.86 | BELOW FLOOR |
| P vs M | q2_p50_us | 185.1 | 21.2 | 195.4 | 37.0 | 30.1 | 15.0 | +5.5% | 0.34 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 195.8 | 9.67 | 182.4 | 6.68 | 8.31 | 107.5 | -6.8% | 1.61 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q4_p50_us | 181.6 | 16.3 | 192.0 | 37.1 | 28.6 | 19.3 | +5.7% | 0.36 | BELOW FLOOR |
| P vs M | q5_p50_us | 121.2 | 12.9 | 114.7 | 9.33 | 11.3 | 26.9 | -5.4% | 0.58 | BELOW FLOOR |
| P vs M | q6_p50_us | 489.4 | 45.9 | 546.4 | 44.4 | 45.1 | 86.1 | +11.6% | 1.26 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 49.5 | 24.7 | 58.2 | 34.1 | 29.8 | 60.7 | +17.7% | 0.29 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 1271 | 313.0 | 1437 | 304.8 | 308.9 | 605.9 | +13.1% | 0.54 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 110.9 | 27.4 | 133.8 | 68.7 | 52.3 | 59.9 | +20.6% | 0.44 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 104.8 | 12.6 | 108.3 | 43.3 | 31.9 | 19.0 | +3.3% | 0.11 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2225 | 244.5 | 1942 | 236.6 | 240.6 | 709.0 | -12.7% | 1.17 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 212.1 | 36.2 | 201.5 | 61.5 | 50.5 | 55.6 | -5.0% | 0.21 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2360 | 531.1 | 2362 | 234.2 | 410.4 | 1371 | +0.1% | 0.01 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 12632 | 307.3 | 12959 | 425.4 | 371.1 | 333.8 | +2.6% | 0.88 | BELOW FLOOR |
| P vs M | write_p99_us | 18289 | 1053 | 18618 | 1927 | 1553 | 3451 | +1.8% | 0.21 | BELOW FLOOR |
| P vs M | commits_per_s | 76.0 | 2.22 | 74.2 | 2.61 | 2.42 | 8.90 | -2.3% | 0.74 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 263.6 | 2.72 | 263.6 | 2.72 | 2.72 | 196.0 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.1 | 0.03 | 44.2 | 0.02 | 0.02 | 0.13 | +0.2% | 4.78 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2629 | 135.6 | -2516 | 109.6 | 123.3 | 2744 | -4.3% | 0.92 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 105.0 | 7.65 | 242.3 | 19.5 | 14.8 | 24.6 | +130.8% | 9.28 | **N better** |
| N vs H3 | q1_p99_us | 327.3 | 63.0 | 687.5 | 74.9 | 69.2 | 543.6 | +110.0% | 5.21 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 126.7 | 14.0 | 264.1 | 20.3 | 17.5 | 97.9 | +108.5% | 7.87 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 132.7 | 15.2 | 296.7 | 38.9 | 29.5 | 73.4 | +123.6% | 5.55 | **N better** |
| N vs H3 | q4_p50_us | 2105 | 19.3 | 1411 | 64.6 | 47.7 | 124.7 | -33.0% | 14.6 | **H3 better** |
| N vs H3 | q5_p50_us | 2386 | 56.9 | 3182 | 84.1 | 71.8 | 226.5 | +33.4% | 11.1 | **N better** |
| N vs H3 | q6_p50_us | 3491 | 184.7 | 3735 | 265.4 | 228.6 | 865.3 | +7.0% | 1.07 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 64.5 | 10.9 | 190.9 | 19.0 | 15.5 | 35.6 | +195.9% | 8.17 | **N better** |
| N vs H3 | c2_q1_p99_us | 312.0 | 70.2 | 728.7 | 107.7 | 90.9 | 604.8 | +133.6% | 4.59 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 81.2 | 13.4 | 222.2 | 31.9 | 24.5 | 66.5 | +173.7% | 5.76 | **N better** |
| N vs H3 | c4_q1_p50_us | 113.7 | 13.0 | 341.9 | 36.0 | 27.1 | 11.9 | +200.8% | 8.43 | **N better** |
| N vs H3 | c4_q1_p99_us | 624.6 | 169.9 | 1176 | 227.0 | 200.5 | 1750 | +88.3% | 2.75 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 134.7 | 22.6 | 389.0 | 26.8 | 24.8 | 101.8 | +188.8% | 10.3 | **N better** |
| N vs H3 | c4_q2_p99_us | 767.3 | 169.1 | 1197 | 173.6 | 171.4 | 842.7 | +56.0% | 2.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 456.3 | 27.7 | 1207 | 85.6 | 63.6 | 131.6 | +164.4% | 11.8 | **N better** |
| N vs H3 | write_p99_us | 755.2 | 33.7 | 3664 | 369.2 | 262.1 | 6864 | +385.1% | 11.1 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 2107 | 78.2 | 716.0 | 53.5 | 67.0 | 55.8 | -66.0% | 20.8 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 329.1 | 2.50 | 110.4 | 0.41 | 1.79 | 160.5 | -66.4% | 122.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 9.86 | 0.21 | 70.5 | 0.10 | 0.17 | 1.20 | +615.2% | 366.5 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 4812 | 209.4 | 16756 | 305.7 | 262.0 | 21642 | +248.2% | 45.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 609.0 | 58.0 | 242.3 | 19.5 | 43.3 | 121.4 | -60.2% | 8.47 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1430 | 294.9 | 687.5 | 74.9 | 215.2 | 1167 | -51.9% | 3.45 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 454.5 | 58.5 | 264.1 | 20.3 | 43.8 | 106.8 | -41.9% | 4.35 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 273.5 | 30.1 | 296.7 | 38.9 | 34.8 | 93.1 | +8.5% | 0.67 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1376 | 139.0 | 1411 | 64.6 | 108.4 | 130.4 | +2.5% | 0.32 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3142 | 152.7 | 3182 | 84.1 | 123.2 | 221.3 | +1.3% | 0.32 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 3262 | 177.7 | 3735 | 265.4 | 225.8 | 655.0 | +14.5% | 2.10 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 738.9 | 130.2 | 190.9 | 19.0 | 93.0 | 210.4 | -74.2% | 5.89 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5679 | 1501 | 728.7 | 107.7 | 1064 | 552.2 | -87.2% | 4.65 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 570.7 | 76.9 | 222.2 | 31.9 | 58.9 | 312.4 | -61.1% | 5.92 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 1734 | 185.1 | 341.9 | 36.0 | 133.3 | 198.5 | -80.3% | 10.4 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 10643 | 2206 | 1176 | 227.0 | 1568 | 2106460 | -88.9% | 6.04 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1594 | 246.3 | 389.0 | 26.8 | 175.2 | 92.2 | -75.6% | 6.88 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8035 | 2157 | 1197 | 173.6 | 1530 | 13356 | -85.1% | 4.47 | BELOW FLOOR |
| P+ vs H3 | write_p50_us | 838.7 | 43.9 | 1207 | 85.6 | 68.0 | 147.9 | +43.9% | 5.41 | **P+ better** |
| P+ vs H3 | write_p99_us | 2568 | 119.2 | 3664 | 369.2 | 274.3 | 6859 | +42.7% | 3.99 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 1058 | 75.7 | 716.0 | 53.5 | 65.6 | 122.7 | -32.3% | 5.21 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 931.5 | 23.2 | 110.4 | 0.41 | 16.4 | 305.0 | -88.1% | 49.9 | **H3 better** |
| P+ vs H3 | pss_mib | 44.0 | 0.03 | 70.5 | 0.10 | 0.08 | 0.13 | +60.3% | 350.4 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2566 | 214.2 | 16756 | 305.7 | 263.9 | 21701 | -752.9% | 73.2 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 197.8 | 19.7 | 608.8 | 12.8 | 16.6 | 63.2 | +207.8% | 24.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 638.1 | 84.2 | 2515 | 904.4 | 642.3 | 217.9 | +294.2% | 2.92 | no difference |
| N vs P+ | q2_p50_us | 173.1 | 12.4 | 477.1 | 33.0 | 24.9 | 43.1 | +175.7% | 12.2 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 161.0 | 19.8 | 288.6 | 19.4 | 19.6 | 22.3 | +79.3% | 6.51 | **N better** |
| N vs P+ | q4_p50_us | 20055 | 541.1 | 10682 | 249.9 | 421.4 | 919.9 | -46.7% | 22.2 | **P+ better** |
| N vs P+ | q5_p50_us | 30243 | 373.6 | 36437 | 443.3 | 409.9 | 1608 | +20.5% | 15.1 | **N better** |
| N vs P+ | q6_p50_us | 48128 | 858.3 | 37221 | 629.9 | 752.8 | 6405 | -22.7% | 14.5 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 160.9 | 35.9 | 674.0 | 94.7 | 71.6 | 12.3 | +318.9% | 7.16 | **N better** |
| N vs P+ | c2_q1_p99_us | 468.9 | 47.9 | 5198 | 1203 | 851.0 | 3318 | +1008.7% | 5.56 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 90.8 | 15.8 | 540.1 | 50.5 | 37.4 | 56.6 | +494.7% | 12.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 235.7 | 24.4 | 1691 | 139.0 | 99.8 | 423.6 | +617.5% | 14.6 | **N better** |
| N vs P+ | c4_q1_p99_us | 1079 | 153.5 | 8724 | 1379 | 981.1 | 853.4 | +708.2% | 7.79 | **N better** |
| N vs P+ | c4_q2_p50_us | 189.0 | 29.0 | 1533 | 49.0 | 40.3 | 814.4 | +710.9% | 33.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p99_us | 1962 | 734.4 | 7371 | 1312 | 1063 | 6032 | +275.7% | 5.09 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 537.2 | 37.8 | 822.5 | 66.4 | 54.1 | 95.2 | +53.1% | 5.28 | **N better** |
| N vs P+ | write_p99_us | 1017 | 294.1 | 2924 | 312.8 | 303.6 | 1999 | +187.5% | 6.28 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1646 | 143.5 | 1018 | 70.0 | 112.9 | 487.8 | -38.2% | 5.56 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 730.8 | 128.1 | 832.0 | 145.8 | 137.3 | 2015 | +13.8% | 0.74 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 39.9 | 1.22 | 43.9 | 0.02 | 0.86 | 0.26 | +10.1% | 4.68 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4173 | 805.8 | -832.5 | 161.0 | 581.0 | 1936 | -120.0% | 8.61 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 197.8 | 19.7 | 102.0 | 1.94 | 14.0 | 38.8 | -48.4% | 6.84 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q1_p99_us | 638.1 | 84.2 | 452.9 | 18.4 | 61.0 | 143.2 | -29.0% | 3.04 | **P better** |
| N vs P | q2_p50_us | 173.1 | 12.4 | 195.0 | 25.1 | 19.8 | 31.5 | +12.7% | 1.11 | BELOW FLOOR |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 161.0 | 19.8 | 244.3 | 33.2 | 27.3 | 11.4 | +51.7% | 3.05 | **N better** |
| N vs P | q4_p50_us | 20055 | 541.1 | 752.0 | 64.1 | 385.3 | 125.7 | -96.3% | 50.1 | **P better** |
| N vs P | q5_p50_us | 30243 | 373.6 | 140.5 | 18.2 | 264.5 | 369.4 | -99.5% | 113.8 | **P better** |
| N vs P | q6_p50_us | 48128 | 858.3 | 2613 | 85.0 | 609.9 | 1866 | -94.6% | 74.6 | **P better** |
| N vs P | c2_q1_p50_us | 160.9 | 35.9 | 70.5 | 37.9 | 36.9 | 3.46 | -56.2% | 2.45 | no difference |
| N vs P | c2_q1_p99_us | 468.9 | 47.9 | 1060 | 202.7 | 147.3 | 535.4 | +126.1% | 4.01 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 90.8 | 15.8 | 168.8 | 58.6 | 42.9 | 52.4 | +85.9% | 1.82 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 235.7 | 24.4 | 83.3 | 15.0 | 20.3 | 71.5 | -64.6% | 7.52 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 1079 | 153.5 | 2469 | 538.0 | 395.6 | 681.7 | +128.7% | 3.51 | **N better** |
| N vs P | c4_q2_p50_us | 189.0 | 29.0 | 220.2 | 30.7 | 29.9 | 248.1 | +16.5% | 1.04 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 1962 | 734.4 | 2668 | 403.5 | 592.5 | 1722 | +36.0% | 1.19 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 537.2 | 37.8 | 114886 | 2836 | 2006 | 8027 | +21287.0% | 57.0 | **N better** |
| N vs P | write_p99_us | 1017 | 294.1 | 158982 | 10500 | 7428 | 24243 | +15533.3% | 21.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1646 | 143.5 | 8.35 | 0.14 | 101.5 | 458.7 | -99.5% | 16.1 | **N better** |
| N vs P | view_state_bytes_per_key_read | 730.8 | 128.1 | 414.0 | 81.8 | 107.5 | 1720 | -43.3% | 2.95 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 39.9 | 1.22 | 44.2 | 0.03 | 0.86 | 0.26 | +10.9% | 5.02 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 4173 | 805.8 | -669.9 | 150.7 | 579.7 | 1646 | -116.1% | 8.35 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 608.8 | 12.8 | 102.0 | 1.94 | 9.15 | 50.2 | -83.2% | 55.4 | **P better** |
| P+ vs P | q1_p99_us | 2515 | 904.4 | 452.9 | 18.4 | 639.6 | 238.4 | -82.0% | 3.22 | **P better** |
| P+ vs P | q2_p50_us | 477.1 | 33.0 | 195.0 | 25.1 | 29.3 | 53.3 | -59.1% | 9.63 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 288.6 | 19.4 | 244.3 | 33.2 | 27.2 | 19.2 | -15.4% | 1.63 | no difference |
| P+ vs P | q4_p50_us | 10682 | 249.9 | 752.0 | 64.1 | 182.4 | 914.9 | -93.0% | 54.4 | **P better** |
| P+ vs P | q5_p50_us | 36437 | 443.3 | 140.5 | 18.2 | 313.7 | 1565 | -99.6% | 115.7 | **P better** |
| P+ vs P | q6_p50_us | 37221 | 629.9 | 2613 | 85.0 | 449.5 | 6166 | -93.0% | 77.0 | **P better** |
| P+ vs P | c2_q1_p50_us | 674.0 | 94.7 | 70.5 | 37.9 | 72.2 | 12.7 | -89.5% | 8.36 | **P better** |
| P+ vs P | c2_q1_p99_us | 5198 | 1203 | 1060 | 202.7 | 862.3 | 3304 | -79.6% | 4.80 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 540.1 | 50.5 | 168.8 | 58.6 | 54.7 | 22.0 | -68.7% | 6.79 | **P better** |
| P+ vs P | c4_q1_p50_us | 1691 | 139.0 | 83.3 | 15.0 | 98.8 | 425.9 | -95.1% | 16.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8724 | 1379 | 2469 | 538.0 | 1047 | 755.3 | -71.7% | 5.98 | **P better** |
| P+ vs P | c4_q2_p50_us | 1533 | 49.0 | 220.2 | 30.7 | 40.9 | 846.1 | -85.6% | 32.1 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p99_us | 7371 | 1312 | 2668 | 403.5 | 970.5 | 5979 | -63.8% | 4.85 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 822.5 | 66.4 | 114886 | 2836 | 2006 | 8027 | +13867.8% | 56.9 | **P+ better** |
| P+ vs P | write_p99_us | 2924 | 312.8 | 158982 | 10500 | 7428 | 24167 | +5336.8% | 21.0 | **P+ better** |
| P+ vs P | commits_per_s | 1018 | 70.0 | 8.35 | 0.14 | 49.5 | 166.1 | -99.2% | 20.4 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 832.0 | 145.8 | 414.0 | 81.8 | 118.2 | 1663 | -50.2% | 3.53 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 43.9 | 0.02 | 44.2 | 0.03 | 0.02 | 0.01 | +0.7% | 13.1 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -832.5 | 161.0 | -669.9 | 150.7 | 155.9 | 2440 | -19.5% | 1.04 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 102.0 | 1.94 | 97.5 | 6.38 | 4.71 | 13.6 | -4.5% | 0.97 | BELOW FLOOR |
| P vs M | q1_p99_us | 452.9 | 18.4 | 466.4 | 66.4 | 48.7 | 308.2 | +3.0% | 0.28 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 195.0 | 25.1 | 208.6 | 18.2 | 21.9 | 49.8 | +7.0% | 0.62 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 244.3 | 33.2 | 227.6 | 48.5 | 41.6 | 89.2 | -6.8% | 0.40 | BELOW FLOOR |
| P vs M | q4_p50_us | 752.0 | 64.1 | 696.9 | 79.2 | 72.0 | 61.7 | -7.3% | 0.76 | BELOW FLOOR |
| P vs M | q5_p50_us | 140.5 | 18.2 | 149.8 | 21.7 | 20.1 | 36.0 | +6.6% | 0.46 | BELOW FLOOR |
| P vs M | q6_p50_us | 2613 | 85.0 | 2582 | 123.5 | 106.0 | 489.1 | -1.2% | 0.29 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 70.5 | 37.9 | 39.1 | 10.8 | 27.8 | 6.49 | -44.5% | 1.13 | no difference |
| P vs M | c2_q1_p99_us | 1060 | 202.7 | 1187 | 195.2 | 199.0 | 332.2 | +11.9% | 0.64 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 168.8 | 58.6 | 140.3 | 33.7 | 47.8 | 44.1 | -16.9% | 0.60 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 83.3 | 15.0 | 87.1 | 17.5 | 16.3 | 66.6 | +4.5% | 0.23 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2469 | 538.0 | 2101 | 292.5 | 433.0 | 659.6 | -14.9% | 0.85 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 220.2 | 30.7 | 218.6 | 24.1 | 27.6 | 280.8 | -0.7% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p99_us | 2668 | 403.5 | 2409 | 535.6 | 474.2 | 1380 | -9.7% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 114886 | 2836 | 115108 | 1549 | 2285 | 8058 | +0.2% | 0.10 | BELOW FLOOR |
| P vs M | write_p99_us | 158982 | 10500 | 159935 | 15586 | 13289 | 25904 | +0.6% | 0.07 | BELOW FLOOR |
| P vs M | commits_per_s | 8.35 | 0.14 | 8.32 | 0.21 | 0.18 | 0.83 | -0.4% | 0.18 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 414.0 | 81.8 | 414.0 | 81.8 | 81.8 | 1291 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.2 | 0.03 | 44.2 | 0.02 | 0.02 | 0.06 | -0.2% | 4.32 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -669.9 | 150.7 | -746.6 | 152.0 | 151.3 | 2331 | +11.4% | 0.51 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 197.8 | 19.7 | 325.6 | 27.3 | 23.8 | 77.9 | +64.6% | 5.37 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 638.1 | 84.2 | 1204 | 378.3 | 274.0 | 513.0 | +88.6% | 2.06 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 173.1 | 12.4 | 299.3 | 30.6 | 23.4 | 28.9 | +72.9% | 5.40 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 161.0 | 19.8 | 283.6 | 40.2 | 31.7 | 18.0 | +76.2% | 3.87 | **N better** |
| N vs H3 | q4_p50_us | 20055 | 541.1 | 11093 | 480.8 | 511.8 | 186.7 | -44.7% | 17.5 | **H3 better** |
| N vs H3 | q5_p50_us | 30243 | 373.6 | 37065 | 1244 | 918.6 | 2281 | +22.6% | 7.43 | **N better** |
| N vs H3 | q6_p50_us | 48128 | 858.3 | 40702 | 1289 | 1095 | 2902 | -15.4% | 6.78 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 160.9 | 35.9 | 308.2 | 34.8 | 35.3 | 94.6 | +91.5% | 4.17 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 468.9 | 47.9 | 1613 | 564.7 | 400.7 | 474.1 | +244.0% | 2.86 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 90.8 | 15.8 | 322.4 | 4.25 | 11.6 | 52.3 | +255.0% | 20.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 235.7 | 24.4 | 540.2 | 16.2 | 20.7 | 246.3 | +129.2% | 14.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 1079 | 153.5 | 2092 | 266.7 | 217.6 | 717.8 | +93.8% | 4.65 | **N better** |
| N vs H3 | c4_q2_p50_us | 189.0 | 29.0 | 569.7 | 47.6 | 39.4 | 141.7 | +201.4% | 9.66 | **N better** |
| N vs H3 | c4_q2_p99_us | 1962 | 734.4 | 2524 | 748.0 | 741.3 | 1343 | +28.6% | 0.76 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 537.2 | 37.8 | 1254 | 51.0 | 44.9 | 167.3 | +133.5% | 16.0 | **N better** |
| N vs H3 | write_p99_us | 1017 | 294.1 | 3733 | 386.5 | 343.4 | 5330 | +267.1% | 7.91 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1646 | 143.5 | 665.6 | 42.3 | 105.8 | 462.1 | -59.6% | 9.27 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 730.8 | 128.1 | 117.1 | 1.76 | 90.6 | 1459 | -84.0% | 6.77 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | pss_mib | 39.9 | 1.22 | 107.6 | 0.31 | 0.89 | 0.74 | +169.6% | 75.9 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 4173 | 805.8 | 19337 | 3339 | 2429 | 45634 | +363.4% | 6.24 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 608.8 | 12.8 | 325.6 | 27.3 | 21.3 | 84.2 | -46.5% | 13.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2515 | 904.4 | 1204 | 378.3 | 693.2 | 547.3 | -52.1% | 1.89 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 477.1 | 33.0 | 299.3 | 30.6 | 31.8 | 51.8 | -37.3% | 5.59 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 288.6 | 19.4 | 283.6 | 40.2 | 31.6 | 23.8 | -1.7% | 0.16 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 10682 | 249.9 | 11093 | 480.8 | 383.2 | 925.3 | +3.8% | 1.07 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 36437 | 443.3 | 37065 | 1244 | 934.0 | 2742 | +1.7% | 0.67 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 37221 | 629.9 | 40702 | 1289 | 1014 | 6555 | +9.4% | 3.43 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 674.0 | 94.7 | 308.2 | 34.8 | 71.4 | 95.4 | -54.3% | 5.13 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 5198 | 1203 | 1613 | 564.7 | 939.4 | 3295 | -69.0% | 3.82 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p50_us | 540.1 | 50.5 | 322.4 | 4.25 | 35.8 | 21.7 | -40.3% | 6.07 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1691 | 139.0 | 540.2 | 16.2 | 98.9 | 486.8 | -68.1% | 11.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 8724 | 1379 | 2092 | 266.7 | 993.1 | 788.1 | -76.0% | 6.68 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1533 | 49.0 | 569.7 | 47.6 | 48.3 | 821.2 | -62.8% | 19.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p99_us | 7371 | 1312 | 2524 | 748.0 | 1068 | 5881 | -65.8% | 4.54 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 822.5 | 66.4 | 1254 | 51.0 | 59.2 | 169.5 | +52.5% | 7.29 | **P+ better** |
| P+ vs H3 | write_p99_us | 2924 | 312.8 | 3733 | 386.5 | 351.6 | 4972 | +27.7% | 2.30 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 1018 | 70.0 | 665.6 | 42.3 | 57.8 | 175.2 | -34.6% | 6.10 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 832.0 | 145.8 | 117.1 | 1.76 | 103.1 | 1391 | -85.9% | 6.93 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | pss_mib | 43.9 | 0.02 | 107.6 | 0.31 | 0.22 | 0.69 | +144.8% | 285.6 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -832.5 | 161.0 | 19337 | 3339 | 2364 | 45669 | -2422.8% | 8.53 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 211.2 | 26.1 | 689.7 | 49.1 | 39.3 | 41.2 | +226.5% | 12.2 | **N better** |
| N vs P+ | q1_p99_us | 762.0 | 91.6 | 1838 | 371.0 | 270.2 | 514.9 | +141.1% | 3.98 | **N better** |
| N vs P+ | q2_p50_us | 180.1 | 8.79 | 519.3 | 50.6 | 36.3 | 63.0 | +188.3% | 9.33 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 169.2 | 26.9 | 316.0 | 50.7 | 40.6 | 98.2 | +86.7% | 3.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | 19833 | 213.3 | 11352 | 672.4 | 498.8 | 1409 | -42.8% | 17.0 | **P+ better** |
| N vs P+ | q5_p50_us | 30194 | 869.5 | 37305 | 1487 | 1218 | 5369 | +23.6% | 5.84 | **N better** |
| N vs P+ | q6_p50_us | 50450 | 3981 | 37986 | 1141 | 2928 | 4357 | -24.7% | 4.26 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 120.5 | 18.4 | 655.8 | 43.0 | 33.1 | 12.8 | +444.2% | 16.2 | **N better** |
| N vs P+ | c2_q1_p99_us | 370.1 | 58.9 | 4780 | 881.1 | 624.4 | 1167 | +1191.4% | 7.06 | **N better** |
| N vs P+ | c2_q2_p50_us | 88.1 | 13.2 | 497.4 | 45.2 | 33.3 | 96.6 | +464.4% | 12.3 | **N better** |
| N vs P+ | c4_q1_p50_us | 246.0 | 37.0 | 1738 | 177.5 | 128.2 | 621.9 | +606.6% | 11.6 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q1_p99_us | 1158 | 263.9 | 8041 | 814.2 | 605.3 | 2784 | +594.1% | 11.4 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 208.3 | 30.2 | 1559 | 125.3 | 91.2 | 447.5 | +648.7% | 14.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1027 | 211.4 | 8344 | 1162 | 835.2 | 1082 | +712.5% | 8.76 | **N better** |
| N vs P+ | write_p50_us | 619.9 | 38.5 | 873.7 | 32.1 | 35.4 | 70.0 | +40.9% | 7.16 | **N better** |
| N vs P+ | write_p99_us | 1482 | 281.3 | 2788 | 328.4 | 305.7 | 2698 | +88.2% | 4.27 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1381 | 140.2 | 961.9 | 78.4 | 113.6 | 674.5 | -30.3% | 3.69 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 733.3 | 126.7 | 848.4 | 151.3 | 139.5 | 1935 | +15.7% | 0.83 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 45.2 | 1.88 | 88.1 | 0.01 | 1.33 | 0.43 | +94.8% | 32.3 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 5504 | 346.2 | -4722 | 987.6 | 740.0 | 22028 | -185.8% | 13.8 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 211.2 | 26.1 | 102.5 | 9.61 | 19.7 | 37.6 | -51.5% | 5.52 | **P better** |
| N vs P | q1_p99_us | 762.0 | 91.6 | 569.8 | 66.5 | 80.0 | 532.7 | -25.2% | 2.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 180.1 | 8.79 | 213.5 | 11.0 | 9.98 | 21.2 | +18.5% | 3.35 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 169.2 | 26.9 | 265.8 | 18.3 | 23.0 | 116.6 | +57.0% | 4.19 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q4_p50_us | 19833 | 213.3 | 733.8 | 68.2 | 158.4 | 1394 | -96.3% | 120.6 | **P better** |
| N vs P | q5_p50_us | 30194 | 869.5 | 173.9 | 33.0 | 615.2 | 1770 | -99.4% | 48.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 50450 | 3981 | 2625 | 166.9 | 2817 | 4284 | -94.8% | 17.0 | **P better** |
| N vs P | c2_q1_p50_us | 120.5 | 18.4 | 89.2 | 23.9 | 21.3 | 52.1 | -26.0% | 1.47 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 370.1 | 58.9 | 1410 | 474.6 | 338.1 | 100.8 | +281.0% | 3.08 | **N better** |
| N vs P | c2_q2_p50_us | 88.1 | 13.2 | 204.9 | 55.8 | 40.5 | 178.8 | +132.5% | 2.88 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 246.0 | 37.0 | 100.6 | 10.7 | 27.3 | 122.6 | -59.1% | 5.33 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1158 | 263.9 | 2099 | 411.6 | 345.8 | 522.4 | +81.2% | 2.72 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 208.3 | 30.2 | 246.0 | 32.6 | 31.4 | 176.6 | +18.1% | 1.20 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 1027 | 211.4 | 2942 | 872.9 | 635.1 | 5915 | +186.4% | 3.01 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 619.9 | 38.5 | 122969 | 4832 | 3417 | 1785 | +19736.7% | 35.8 | **N better** |
| N vs P | write_p99_us | 1482 | 281.3 | 167633 | 5138 | 3638 | 6844 | +11213.2% | 45.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1381 | 140.2 | 7.97 | 0.25 | 99.2 | 674.5 | -99.4% | 13.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | view_state_bytes_per_key_read | 733.3 | 126.7 | 415.5 | 77.8 | 105.1 | 1639 | -43.3% | 3.02 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 45.2 | 1.88 | 88.5 | 0.02 | 1.33 | 0.43 | +95.7% | 32.6 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 5504 | 346.2 | -2367 | 480.3 | 418.7 | 20249 | -143.0% | 18.8 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 689.7 | 49.1 | 102.5 | 9.61 | 35.4 | 41.0 | -85.1% | 16.6 | **P better** |
| P+ vs P | q1_p99_us | 1838 | 371.0 | 569.8 | 66.5 | 266.5 | 732.8 | -69.0% | 4.76 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 519.3 | 50.6 | 213.5 | 11.0 | 36.6 | 62.7 | -58.9% | 8.34 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 316.0 | 50.7 | 265.8 | 18.3 | 38.1 | 124.0 | -15.9% | 1.32 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 11352 | 672.4 | 733.8 | 68.2 | 477.9 | 425.9 | -93.5% | 22.2 | **P better** |
| P+ vs P | q5_p50_us | 37305 | 1487 | 173.9 | 33.0 | 1052 | 5070 | -99.5% | 35.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 37986 | 1141 | 2625 | 166.9 | 815.6 | 806.8 | -93.1% | 43.4 | **P better** |
| P+ vs P | c2_q1_p50_us | 655.8 | 43.0 | 89.2 | 23.9 | 34.8 | 51.1 | -86.4% | 16.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 4780 | 881.1 | 1410 | 474.6 | 707.7 | 1165 | -70.5% | 4.76 | **P better** |
| P+ vs P | c2_q2_p50_us | 497.4 | 45.2 | 204.9 | 55.8 | 50.8 | 199.4 | -58.8% | 5.76 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1738 | 177.5 | 100.6 | 10.7 | 125.8 | 612.7 | -94.2% | 13.0 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p99_us | 8041 | 814.2 | 2099 | 411.6 | 645.2 | 2747 | -73.9% | 9.21 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1559 | 125.3 | 246.0 | 32.6 | 91.6 | 472.0 | -84.2% | 14.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 8344 | 1162 | 2942 | 872.9 | 1028 | 5998 | -64.7% | 5.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 873.7 | 32.1 | 122969 | 4832 | 3417 | 1784 | +13974.5% | 35.7 | **P+ better** |
| P+ vs P | write_p99_us | 2788 | 328.4 | 167633 | 5138 | 3640 | 6324 | +5911.9% | 45.3 | **P+ better** |
| P+ vs P | commits_per_s | 961.9 | 78.4 | 7.97 | 0.25 | 55.4 | 2.29 | -99.2% | 17.2 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 848.4 | 151.3 | 415.5 | 77.8 | 120.3 | 1609 | -51.0% | 3.60 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 88.1 | 0.01 | 88.5 | 0.02 | 0.01 | 0.03 | +0.5% | 30.1 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -4722 | 987.6 | -2367 | 480.3 | 776.6 | 11225 | -49.9% | 3.03 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 102.5 | 9.61 | 109.7 | 9.89 | 9.75 | 31.4 | +7.0% | 0.73 | BELOW FLOOR |
| P vs M | q1_p99_us | 569.8 | 66.5 | 572.1 | 125.8 | 100.6 | 551.7 | +0.4% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 213.5 | 11.0 | 222.0 | 21.2 | 16.9 | 26.2 | +4.0% | 0.50 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 265.8 | 18.3 | 234.5 | 28.0 | 23.7 | 100.7 | -11.8% | 1.32 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q4_p50_us | 733.8 | 68.2 | 792.4 | 78.1 | 73.3 | 423.6 | +8.0% | 0.80 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q5_p50_us | 173.9 | 33.0 | 205.1 | 23.1 | 28.5 | 74.8 | +17.9% | 1.10 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 2625 | 166.9 | 2620 | 157.3 | 162.2 | 89.8 | -0.2% | 0.03 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 89.2 | 23.9 | 90.1 | 38.5 | 32.1 | 58.3 | +1.0% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1410 | 474.6 | 1494 | 382.3 | 430.9 | 443.0 | +5.9% | 0.19 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 204.9 | 55.8 | 204.6 | 37.1 | 47.4 | 187.9 | -0.1% | 0.01 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p50_us | 100.6 | 10.7 | 86.8 | 45.4 | 33.0 | 44.5 | -13.8% | 0.42 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2099 | 411.6 | 2056 | 188.6 | 320.2 | 266.9 | -2.1% | 0.13 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 246.0 | 32.6 | 211.1 | 70.4 | 54.8 | 163.9 | -14.2% | 0.64 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 2942 | 872.9 | 2800 | 451.4 | 694.9 | 6647 | -4.8% | 0.20 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 122969 | 4832 | 122563 | 1505 | 3579 | 9107 | -0.3% | 0.11 | BELOW FLOOR |
| P vs M | write_p99_us | 167633 | 5138 | 155769 | 7100 | 6197 | 31410 | -7.1% | 1.91 | BELOW FLOOR |
| P vs M | commits_per_s | 7.97 | 0.25 | 7.97 | 0.10 | 0.19 | 1.36 | +0.0% | 0.01 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 415.5 | 77.8 | 415.5 | 77.8 | 77.8 | 1238 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 88.5 | 0.02 | 88.8 | 0.02 | 0.02 | 0.12 | +0.2% | 14.3 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2367 | 480.3 | -1360 | 267.4 | 388.7 | 5865 | -42.5% | 2.59 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 211.2 | 26.1 | 347.4 | 17.2 | 22.1 | 28.4 | +64.5% | 6.16 | **N better** |
| N vs H3 | q1_p99_us | 762.0 | 91.6 | 1128 | 242.4 | 183.2 | 90.1 | +48.0% | 2.00 | no difference |
| N vs H3 | q2_p50_us | 180.1 | 8.79 | 371.4 | 27.0 | 20.1 | 25.1 | +106.2% | 9.53 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 169.2 | 26.9 | 350.1 | 49.4 | 39.8 | 70.5 | +106.9% | 4.55 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | 19833 | 213.3 | 11672 | 338.7 | 283.1 | 7096 | -41.1% | 28.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q5_p50_us | 30194 | 869.5 | 38092 | 1923 | 1492 | 1885 | +26.2% | 5.29 | **N better** |
| N vs H3 | q6_p50_us | 50450 | 3981 | 41884 | 2283 | 3245 | 11331 | -17.0% | 2.64 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 120.5 | 18.4 | 299.7 | 37.8 | 29.8 | 46.4 | +148.7% | 6.02 | **N better** |
| N vs H3 | c2_q1_p99_us | 370.1 | 58.9 | 1963 | 520.1 | 370.1 | 1677 | +430.4% | 4.30 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 88.1 | 13.2 | 329.7 | 59.9 | 43.4 | 55.0 | +274.2% | 5.57 | **N better** |
| N vs H3 | c4_q1_p50_us | 246.0 | 37.0 | 558.9 | 53.5 | 46.0 | 138.4 | +127.2% | 6.80 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 1158 | 263.9 | 2097 | 222.7 | 244.2 | 1856 | +81.0% | 3.84 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 208.3 | 30.2 | 544.9 | 40.1 | 35.5 | 134.8 | +161.6% | 9.48 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1027 | 211.4 | 2585 | 712.3 | 525.4 | 2641 | +151.7% | 2.97 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 619.9 | 38.5 | 1326 | 65.9 | 54.0 | 66.7 | +113.8% | 13.1 | **N better** |
| N vs H3 | write_p99_us | 1482 | 281.3 | 4458 | 1046 | 766.1 | 2671 | +200.9% | 3.89 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1381 | 140.2 | 646.0 | 55.4 | 106.6 | 675.1 | -53.2% | 6.89 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 733.3 | 126.7 | 118.5 | 4.06 | 89.6 | 1386 | -83.8% | 6.86 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 45.2 | 1.88 | 108.9 | 0.33 | 1.35 | 0.44 | +140.6% | 47.2 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 5504 | 346.2 | 5175 | 452.2 | 402.7 | 21967 | -6.0% | 0.82 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 689.7 | 49.1 | 347.4 | 17.2 | 36.8 | 32.7 | -49.6% | 9.31 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1838 | 371.0 | 1128 | 242.4 | 313.4 | 511.1 | -38.6% | 2.26 | no difference |
| P+ vs H3 | q2_p50_us | 519.3 | 50.6 | 371.4 | 27.0 | 40.6 | 64.1 | -28.5% | 3.65 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 316.0 | 50.7 | 350.1 | 49.4 | 50.1 | 82.2 | +10.8% | 0.68 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 11352 | 672.4 | 11672 | 338.7 | 532.4 | 6971 | +2.8% | 0.60 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q5_p50_us | 37305 | 1487 | 38092 | 1923 | 1719 | 5111 | +2.1% | 0.46 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 37986 | 1141 | 41884 | 2283 | 1805 | 10521 | +10.3% | 2.16 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 655.8 | 43.0 | 299.7 | 37.8 | 40.5 | 45.3 | -54.3% | 8.79 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4780 | 881.1 | 1963 | 520.1 | 723.5 | 2039 | -58.9% | 3.89 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 497.4 | 45.2 | 329.7 | 59.9 | 53.1 | 104.0 | -33.7% | 3.16 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1738 | 177.5 | 558.9 | 53.5 | 131.1 | 616.1 | -67.9% | 9.00 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p99_us | 8041 | 814.2 | 2097 | 222.7 | 596.9 | 3274 | -73.9% | 9.96 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1559 | 125.3 | 544.9 | 40.1 | 93.1 | 458.0 | -65.1% | 10.9 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8344 | 1162 | 2585 | 712.3 | 963.8 | 2824 | -69.0% | 5.97 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 873.7 | 32.1 | 1326 | 65.9 | 51.8 | 21.5 | +51.7% | 8.72 | **P+ better** |
| P+ vs H3 | write_p99_us | 2788 | 328.4 | 4458 | 1046 | 775.4 | 529.8 | +59.9% | 2.15 | no difference |
| P+ vs H3 | commits_per_s | 961.9 | 78.4 | 646.0 | 55.4 | 67.9 | 27.9 | -32.8% | 4.66 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 848.4 | 151.3 | 118.5 | 4.06 | 107.0 | 1351 | -86.0% | 6.82 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 88.1 | 0.01 | 108.9 | 0.33 | 0.24 | 0.10 | +23.6% | 87.5 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -4722 | 987.6 | 5175 | 452.2 | 768.1 | 14090 | -209.6% | 12.9 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 180.9 | 16.6 | 651.5 | 42.6 | 32.3 | 110.1 | +260.2% | 14.6 | **N better** |
| N vs P+ | q1_p99_us | 647.8 | 101.4 | 1835 | 429.2 | 311.8 | 1420 | +183.3% | 3.81 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 144.0 | 10.7 | 487.6 | 39.7 | 29.0 | 190.0 | +238.7% | 11.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 144.4 | 16.2 | 294.8 | 17.9 | 17.1 | 48.6 | +104.1% | 8.81 | **N better** |
| N vs P+ | q4_p50_us | 20012 | 167.4 | 11409 | 261.1 | 219.3 | 1891 | -43.0% | 39.2 | **P+ better** |
| N vs P+ | q5_p50_us | 30071 | 524.4 | 37053 | 2053 | 1498 | 8833 | +23.2% | 4.66 | BELOW FLOOR |
| N vs P+ | q6_p50_us | 48708 | 1449 | 40231 | 3309 | 2554 | 8318 | -17.4% | 3.32 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 116.5 | 11.1 | 673.2 | 61.6 | 44.3 | 55.4 | +478.0% | 12.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 541.8 | 164.9 | 5143 | 503.7 | 374.8 | 202.2 | +849.3% | 12.3 | **N better** |
| N vs P+ | c2_q2_p50_us | 83.7 | 8.53 | 528.9 | 33.3 | 24.3 | 73.3 | +531.8% | 18.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 204.8 | 30.1 | 1765 | 72.0 | 55.2 | 80.2 | +761.8% | 28.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1176 | 164.5 | 10770 | 1441 | 1026 | 1621 | +815.5% | 9.35 | **N better** |
| N vs P+ | c4_q2_p50_us | 180.5 | 24.1 | 1517 | 95.5 | 69.6 | 149.1 | +740.6% | 19.2 | **N better** |
| N vs P+ | c4_q2_p99_us | 1615 | 529.3 | 7494 | 2543 | 1837 | 6085 | +364.0% | 3.20 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 604.9 | 48.9 | 906.1 | 110.5 | 85.5 | 86.9 | +49.8% | 3.52 | **N better** |
| N vs P+ | write_p99_us | 1337 | 318.7 | 2831 | 401.3 | 362.3 | 900.0 | +111.7% | 4.12 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1547 | 126.5 | 955.1 | 52.7 | 96.9 | 324.2 | -38.3% | 6.11 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 741.3 | 130.5 | 846.1 | 150.7 | 141.0 | 1935 | +14.1% | 0.74 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 45.3 | 1.66 | 45.0 | 0.02 | 1.17 | 0.53 | -0.6% | 0.25 | BELOW FLOOR |
| N vs P+ | pss_growth_bytes_per_key_read | 5628 | 248.8 | -742.7 | 147.3 | 204.5 | 3261 | -113.2% | 31.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 180.9 | 16.6 | 103.3 | 11.3 | 14.2 | 4.96 | -42.9% | 5.46 | **P better** |
| N vs P | q1_p99_us | 647.8 | 101.4 | 506.4 | 59.5 | 83.1 | 59.9 | -21.8% | 1.70 | no difference |
| N vs P | q2_p50_us | 144.0 | 10.7 | 212.7 | 23.7 | 18.4 | 11.8 | +47.8% | 3.74 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 144.4 | 16.2 | 251.1 | 54.8 | 40.4 | 177.0 | +73.9% | 2.64 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q4_p50_us | 20012 | 167.4 | 759.5 | 62.8 | 126.4 | 762.5 | -96.2% | 152.3 | **P better** |
| N vs P | q5_p50_us | 30071 | 524.4 | 188.9 | 35.0 | 371.6 | 8456 | -99.4% | 80.4 | **P better** |
| N vs P | q6_p50_us | 48708 | 1449 | 2836 | 322.9 | 1050 | 8286 | -94.2% | 43.7 | **P better** |
| N vs P | c2_q1_p50_us | 116.5 | 11.1 | 71.2 | 30.6 | 23.0 | 35.2 | -38.9% | 1.97 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 541.8 | 164.9 | 1311 | 560.7 | 413.2 | 527.9 | +141.9% | 1.86 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 83.7 | 8.53 | 157.3 | 47.0 | 33.8 | 132.8 | +87.9% | 2.18 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p50_us | 204.8 | 30.1 | 119.6 | 31.7 | 30.9 | 59.8 | -41.6% | 2.76 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 1176 | 164.5 | 2863 | 691.5 | 502.6 | 477.7 | +143.4% | 3.36 | **N better** |
| N vs P | c4_q2_p50_us | 180.5 | 24.1 | 283.3 | 58.6 | 44.8 | 71.2 | +57.0% | 2.30 | no difference |
| N vs P | c4_q2_p99_us | 1615 | 529.3 | 4261 | 1803 | 1329 | 1474 | +163.8% | 1.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 604.9 | 48.9 | 116466 | 2874 | 2032 | 5512 | +19152.3% | 57.0 | **N better** |
| N vs P | write_p99_us | 1337 | 318.7 | 161213 | 12587 | 8903 | 40448 | +11956.1% | 18.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1547 | 126.5 | 8.24 | 0.35 | 89.5 | 302.9 | -99.5% | 17.2 | **N better** |
| N vs P | view_state_bytes_per_key_read | 741.3 | 130.5 | 419.7 | 81.9 | 109.0 | 1633 | -43.4% | 2.95 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 45.3 | 1.66 | 44.9 | 0.03 | 1.17 | 0.53 | -0.9% | 0.34 | BELOW FLOOR |
| N vs P | pss_growth_bytes_per_key_read | 5628 | 248.8 | -781.9 | 192.2 | 222.3 | 3281 | -113.9% | 28.8 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 651.5 | 42.6 | 103.3 | 11.3 | 31.2 | 110.2 | -84.1% | 17.6 | **P better** |
| P+ vs P | q1_p99_us | 1835 | 429.2 | 506.4 | 59.5 | 306.4 | 1422 | -72.4% | 4.34 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 487.6 | 39.7 | 212.7 | 23.7 | 32.7 | 190.2 | -56.4% | 8.41 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 294.8 | 17.9 | 251.1 | 54.8 | 40.8 | 173.7 | -14.8% | 1.07 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 11409 | 261.1 | 759.5 | 62.8 | 189.9 | 1730 | -93.3% | 56.1 | **P better** |
| P+ vs P | q5_p50_us | 37053 | 2053 | 188.9 | 35.0 | 1452 | 2552 | -99.5% | 25.4 | **P better** |
| P+ vs P | q6_p50_us | 40231 | 3309 | 2836 | 322.9 | 2351 | 731.7 | -93.0% | 15.9 | **P better** |
| P+ vs P | c2_q1_p50_us | 673.2 | 61.6 | 71.2 | 30.6 | 48.7 | 44.3 | -89.4% | 12.4 | **P better** |
| P+ vs P | c2_q1_p99_us | 5143 | 503.7 | 1311 | 560.7 | 532.9 | 560.6 | -74.5% | 7.19 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 528.9 | 33.3 | 157.3 | 47.0 | 40.7 | 128.4 | -70.3% | 9.13 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1765 | 72.0 | 119.6 | 31.7 | 55.6 | 57.7 | -93.2% | 29.6 | **P better** |
| P+ vs P | c4_q1_p99_us | 10770 | 1441 | 2863 | 691.5 | 1131 | 1659 | -73.4% | 6.99 | **P better** |
| P+ vs P | c4_q2_p50_us | 1517 | 95.5 | 283.3 | 58.6 | 79.2 | 163.9 | -81.3% | 15.6 | **P better** |
| P+ vs P | c4_q2_p99_us | 7494 | 2543 | 4261 | 1803 | 2205 | 5911 | -43.1% | 1.47 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 906.1 | 110.5 | 116466 | 2874 | 2034 | 5513 | +12752.9% | 56.8 | **P+ better** |
| P+ vs P | write_p99_us | 2831 | 401.3 | 161213 | 12587 | 8905 | 40439 | +5595.5% | 17.8 | **P+ better** |
| P+ vs P | commits_per_s | 955.1 | 52.7 | 8.24 | 0.35 | 37.3 | 115.5 | -99.1% | 25.4 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 846.1 | 150.7 | 419.7 | 81.9 | 121.3 | 1608 | -50.4% | 3.51 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 45.0 | 0.02 | 44.9 | 0.03 | 0.03 | 0.20 | -0.2% | 3.77 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -742.7 | 147.3 | -781.9 | 192.2 | 171.2 | 2398 | +5.3% | 0.23 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 103.3 | 11.3 | 107.1 | 9.88 | 10.6 | 6.14 | +3.6% | 0.36 | BELOW FLOOR |
| P vs M | q1_p99_us | 506.4 | 59.5 | 532.9 | 48.7 | 54.3 | 260.0 | +5.2% | 0.49 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 212.7 | 23.7 | 194.1 | 11.8 | 18.7 | 31.9 | -8.8% | 1.00 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 251.1 | 54.8 | 258.3 | 20.5 | 41.4 | 220.1 | +2.8% | 0.17 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 759.5 | 62.8 | 748.7 | 44.9 | 54.6 | 27.2 | -1.4% | 0.20 | BELOW FLOOR |
| P vs M | q5_p50_us | 188.9 | 35.0 | 148.3 | 23.2 | 29.7 | 62.2 | -21.5% | 1.37 | BELOW FLOOR |
| P vs M | q6_p50_us | 2836 | 322.9 | 2770 | 327.2 | 325.1 | 37.7 | -2.3% | 0.20 | no difference |
| P vs M | c2_q1_p50_us | 71.2 | 30.6 | 103.0 | 16.7 | 24.7 | 12.1 | +44.6% | 1.29 | no difference |
| P vs M | c2_q1_p99_us | 1311 | 560.7 | 1591 | 317.5 | 455.6 | 1270 | +21.4% | 0.61 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 157.3 | 47.0 | 235.7 | 56.1 | 51.7 | 202.6 | +49.9% | 1.52 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p50_us | 119.6 | 31.7 | 129.0 | 30.3 | 31.0 | 24.4 | +7.9% | 0.31 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2863 | 691.5 | 2698 | 282.5 | 528.2 | 1450 | -5.8% | 0.31 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 283.3 | 58.6 | 288.3 | 79.2 | 69.7 | 78.6 | +1.8% | 0.07 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 4261 | 1803 | 3851 | 817.4 | 1400 | 442.8 | -9.6% | 0.29 | BELOW FLOOR |
| P vs M | write_p50_us | 116466 | 2874 | 118536 | 2768 | 2822 | 5560 | +1.8% | 0.73 | BELOW FLOOR |
| P vs M | write_p99_us | 161213 | 12587 | 156484 | 9285 | 11060 | 50626 | -2.9% | 0.43 | BELOW FLOOR |
| P vs M | commits_per_s | 8.24 | 0.35 | 8.12 | 0.18 | 0.28 | 0.70 | -1.5% | 0.43 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 419.7 | 81.9 | 419.7 | 81.9 | 81.9 | 1228 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.9 | 0.03 | 44.9 | 0.01 | 0.02 | 0.17 | -0.1% | 1.49 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -781.9 | 192.2 | -752.1 | 145.6 | 170.5 | 2413 | -3.8% | 0.17 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 180.9 | 16.6 | 354.0 | 78.6 | 56.8 | 95.7 | +95.7% | 3.05 | **N better** |
| N vs H3 | q1_p99_us | 647.8 | 101.4 | 1177 | 249.3 | 190.3 | 487.9 | +81.7% | 2.78 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 144.0 | 10.7 | 328.4 | 52.8 | 38.1 | 10.4 | +128.1% | 4.84 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 144.4 | 16.2 | 347.9 | 46.3 | 34.7 | 114.9 | +140.9% | 5.87 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q4_p50_us | 20012 | 167.4 | 12265 | 1077 | 770.4 | 1271 | -38.7% | 10.1 | **H3 better** |
| N vs H3 | q5_p50_us | 30071 | 524.4 | 37730 | 1099 | 861.0 | 17344 | +25.5% | 8.90 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q6_p50_us | 48708 | 1449 | 41843 | 2399 | 1982 | 12829 | -14.1% | 3.46 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 116.5 | 11.1 | 316.1 | 30.0 | 22.6 | 62.3 | +171.4% | 8.84 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 541.8 | 164.9 | 1395 | 242.7 | 207.5 | 1597 | +157.5% | 4.11 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 83.7 | 8.53 | 316.2 | 37.4 | 27.1 | 58.3 | +277.7% | 8.58 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 204.8 | 30.1 | 531.4 | 38.6 | 34.6 | 125.0 | +159.4% | 9.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 1176 | 164.5 | 2212 | 371.4 | 287.3 | 1850 | +88.0% | 3.60 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 180.5 | 24.1 | 583.8 | 55.1 | 42.5 | 22.0 | +223.5% | 9.48 | **N better** |
| N vs H3 | c4_q2_p99_us | 1615 | 529.3 | 2669 | 463.2 | 497.3 | 1902 | +65.3% | 2.12 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 604.9 | 48.9 | 1341 | 91.5 | 73.4 | 136.4 | +121.6% | 10.0 | **N better** |
| N vs H3 | write_p99_us | 1337 | 318.7 | 3939 | 292.3 | 305.8 | 2253 | +194.5% | 8.51 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1547 | 126.5 | 635.6 | 55.5 | 97.7 | 314.8 | -58.9% | 9.33 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 741.3 | 130.5 | 118.5 | 3.27 | 92.3 | 1383 | -84.0% | 6.75 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 45.3 | 1.66 | 116.2 | 0.33 | 1.20 | 0.59 | +156.5% | 59.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 5628 | 248.8 | 22376 | 3886 | 2753 | 43693 | +297.6% | 6.08 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 651.5 | 42.6 | 354.0 | 78.6 | 63.2 | 145.9 | -45.7% | 4.71 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1835 | 429.2 | 1177 | 249.3 | 351.0 | 1502 | -35.9% | 1.87 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 487.6 | 39.7 | 328.4 | 52.8 | 46.7 | 190.1 | -32.6% | 3.41 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 294.8 | 17.9 | 347.9 | 46.3 | 35.1 | 109.9 | +18.0% | 1.51 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 11409 | 261.1 | 12265 | 1077 | 783.3 | 2007 | +7.5% | 1.09 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 37053 | 2053 | 37730 | 1099 | 1646 | 15357 | +1.8% | 0.41 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q6_p50_us | 40231 | 3309 | 41843 | 2399 | 2890 | 9822 | +4.0% | 0.56 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 673.2 | 61.6 | 316.1 | 30.0 | 48.5 | 67.9 | -53.0% | 7.37 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5143 | 503.7 | 1395 | 242.7 | 395.3 | 1609 | -72.9% | 9.48 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 528.9 | 33.3 | 316.2 | 37.4 | 35.4 | 47.6 | -40.2% | 6.01 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1765 | 72.0 | 531.4 | 38.6 | 57.8 | 124.0 | -69.9% | 21.4 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 10770 | 1441 | 2212 | 371.4 | 1053 | 2439 | -79.5% | 8.13 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1517 | 95.5 | 583.8 | 55.1 | 77.9 | 149.2 | -61.5% | 12.0 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7494 | 2543 | 2669 | 463.2 | 1828 | 6032 | -64.4% | 2.64 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 906.1 | 110.5 | 1341 | 91.5 | 101.5 | 160.8 | +47.9% | 4.28 | **P+ better** |
| P+ vs H3 | write_p99_us | 2831 | 401.3 | 3939 | 292.3 | 351.0 | 2092 | +39.1% | 3.16 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 955.1 | 52.7 | 635.6 | 55.5 | 54.1 | 143.8 | -33.5% | 5.91 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 846.1 | 150.7 | 118.5 | 3.27 | 106.6 | 1354 | -86.0% | 6.82 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 45.0 | 0.02 | 116.2 | 0.33 | 0.23 | 0.32 | +158.2% | 307.4 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -742.7 | 147.3 | 22376 | 3886 | 2750 | 43636 | -3113.0% | 8.41 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 188.0 | 19.7 | 621.0 | 31.0 | 26.0 | 45.3 | +230.2% | 16.7 | **N better** |
| N vs P+ | q1_p99_us | 627.7 | 90.6 | 1674 | 282.5 | 209.8 | 1322 | +166.7% | 4.99 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 145.8 | 10.7 | 499.5 | 36.9 | 27.2 | 36.1 | +242.5% | 13.0 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 187.6 | 30.6 | 305.7 | 20.3 | 26.0 | 133.2 | +63.0% | 4.54 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q4_p50_us | 19884 | 334.8 | 11528 | 556.2 | 459.0 | 2513 | -42.0% | 18.2 | **P+ better** |
| N vs P+ | q5_p50_us | 30710 | 771.9 | 37835 | 1201 | 1010 | 1347 | +23.2% | 7.06 | **N better** |
| N vs P+ | q6_p50_us | 48110 | 1332 | 39852 | 1907 | 1645 | 14284 | -17.2% | 5.02 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 119.8 | 19.1 | 688.2 | 79.6 | 57.9 | 70.0 | +474.3% | 9.82 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 642.3 | 226.2 | 6370 | 825.7 | 605.3 | 1100 | +891.8% | 9.46 | **N better** |
| N vs P+ | c2_q2_p50_us | 101.7 | 11.3 | 550.6 | 50.2 | 36.4 | 35.7 | +441.2% | 12.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 238.8 | 45.8 | 1932 | 85.6 | 68.6 | 184.0 | +709.1% | 24.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1086 | 261.2 | 8880 | 598.6 | 461.8 | 803.0 | +717.3% | 16.9 | **N better** |
| N vs P+ | c4_q2_p50_us | 186.8 | 29.4 | 1653 | 88.7 | 66.1 | 72.3 | +784.8% | 22.2 | **N better** |
| N vs P+ | c4_q2_p99_us | 1286 | 226.3 | 6442 | 856.5 | 626.4 | 2931 | +400.7% | 8.23 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 560.9 | 24.1 | 847.1 | 38.6 | 32.2 | 76.7 | +51.0% | 8.90 | **N better** |
| N vs P+ | write_p99_us | 1036 | 139.8 | 2812 | 368.4 | 278.6 | 445.0 | +171.3% | 6.37 | **N better** |
| N vs P+ | commits_per_s | 1573 | 154.3 | 973.4 | 73.4 | 120.8 | 314.5 | -38.1% | 4.96 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 744.8 | 135.1 | 852.0 | 155.0 | 145.4 | 2024 | +14.4% | 0.74 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 44.3 | 0.84 | 44.7 | 0.02 | 0.60 | 2.36 | +1.0% | 0.73 | BELOW FLOOR |
| N vs P+ | pss_growth_bytes_per_key_read | 4986 | 433.4 | -850.8 | 167.6 | 328.6 | 13657 | -117.1% | 17.8 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 188.0 | 19.7 | 99.5 | 10.4 | 15.7 | 5.30 | -47.1% | 5.63 | **P better** |
| N vs P | q1_p99_us | 627.7 | 90.6 | 499.1 | 65.7 | 79.1 | 179.7 | -20.5% | 1.63 | BELOW FLOOR |
| N vs P | q2_p50_us | 145.8 | 10.7 | 202.9 | 13.9 | 12.4 | 37.3 | +39.2% | 4.59 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 187.6 | 30.6 | 265.0 | 43.3 | 37.5 | 113.4 | +41.3% | 2.06 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q4_p50_us | 19884 | 334.8 | 682.4 | 31.2 | 237.7 | 2362 | -96.6% | 80.8 | **P better** |
| N vs P | q5_p50_us | 30710 | 771.9 | 156.1 | 19.2 | 546.0 | 1347 | -99.5% | 56.0 | **P better** |
| N vs P | q6_p50_us | 48110 | 1332 | 2678 | 156.8 | 948.5 | 14279 | -94.4% | 47.9 | **P better** |
| N vs P | c2_q1_p50_us | 119.8 | 19.1 | 55.1 | 26.2 | 22.9 | 34.2 | -54.0% | 2.83 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 642.3 | 226.2 | 1518 | 307.8 | 270.1 | 1213 | +136.3% | 3.24 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 101.7 | 11.3 | 164.1 | 40.0 | 29.4 | 37.6 | +61.3% | 2.12 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 238.8 | 45.8 | 111.6 | 44.6 | 45.2 | 64.6 | -53.3% | 2.81 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1086 | 261.2 | 2297 | 220.6 | 241.8 | 291.0 | +111.4% | 5.01 | **N better** |
| N vs P | c4_q2_p50_us | 186.8 | 29.4 | 239.2 | 53.3 | 43.1 | 58.0 | +28.1% | 1.22 | BELOW FLOOR |
| N vs P | c4_q2_p99_us | 1286 | 226.3 | 2860 | 942.6 | 685.4 | 2227 | +122.3% | 2.29 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 560.9 | 24.1 | 115774 | 1789 | 1265 | 5924 | +20540.1% | 91.0 | **N better** |
| N vs P | write_p99_us | 1036 | 139.8 | 153063 | 8309 | 5876 | 1231 | +14669.3% | 25.9 | **N better** |
| N vs P | commits_per_s | 1573 | 154.3 | 8.37 | 0.14 | 109.1 | 296.8 | -99.5% | 14.3 | **N better** |
| N vs P | view_state_bytes_per_key_read | 744.8 | 135.1 | 421.3 | 84.9 | 112.8 | 1711 | -43.4% | 2.87 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 44.3 | 0.84 | 44.7 | 0.03 | 0.60 | 2.36 | +1.1% | 0.81 | BELOW FLOOR |
| N vs P | pss_growth_bytes_per_key_read | 4986 | 433.4 | -835.9 | 175.8 | 330.7 | 13650 | -116.8% | 17.6 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 621.0 | 31.0 | 99.5 | 10.4 | 23.1 | 45.1 | -84.0% | 22.5 | **P better** |
| P+ vs P | q1_p99_us | 1674 | 282.5 | 499.1 | 65.7 | 205.1 | 1315 | -70.2% | 5.73 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 499.5 | 36.9 | 202.9 | 13.9 | 27.9 | 50.1 | -59.4% | 10.6 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 305.7 | 20.3 | 265.0 | 43.3 | 33.8 | 106.8 | -13.3% | 1.20 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q4_p50_us | 11528 | 556.2 | 682.4 | 31.2 | 393.9 | 857.7 | -94.1% | 27.5 | **P better** |
| P+ vs P | q5_p50_us | 37835 | 1201 | 156.1 | 19.2 | 849.5 | 30.6 | -99.6% | 44.4 | **P better** |
| P+ vs P | q6_p50_us | 39852 | 1907 | 2678 | 156.8 | 1353 | 531.8 | -93.3% | 27.5 | **P better** |
| P+ vs P | c2_q1_p50_us | 688.2 | 79.6 | 55.1 | 26.2 | 59.2 | 61.4 | -92.0% | 10.7 | **P better** |
| P+ vs P | c2_q1_p99_us | 6370 | 825.7 | 1518 | 307.8 | 623.1 | 1638 | -76.2% | 7.79 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 550.6 | 50.2 | 164.1 | 40.0 | 45.4 | 16.7 | -70.2% | 8.52 | **P better** |
| P+ vs P | c4_q1_p50_us | 1932 | 85.6 | 111.6 | 44.6 | 68.2 | 184.4 | -94.2% | 26.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8880 | 598.6 | 2297 | 220.6 | 451.1 | 853.3 | -74.1% | 14.6 | **P better** |
| P+ vs P | c4_q2_p50_us | 1653 | 88.7 | 239.2 | 53.3 | 73.2 | 90.8 | -85.5% | 19.3 | **P better** |
| P+ vs P | c4_q2_p99_us | 6442 | 856.5 | 2860 | 942.6 | 900.6 | 2039 | -55.6% | 3.98 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 847.1 | 38.6 | 115774 | 1789 | 1266 | 5925 | +13567.6% | 90.8 | **P+ better** |
| P+ vs P | write_p99_us | 2812 | 368.4 | 153063 | 8309 | 5881 | 1175 | +5343.3% | 25.5 | **P+ better** |
| P+ vs P | commits_per_s | 973.4 | 73.4 | 8.37 | 0.14 | 51.9 | 104.2 | -99.1% | 18.6 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 852.0 | 155.0 | 421.3 | 84.9 | 125.0 | 1686 | -50.6% | 3.45 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.7 | 0.02 | 44.7 | 0.03 | 0.02 | 0.08 | +0.1% | 2.11 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -850.8 | 167.6 | -835.9 | 175.8 | 171.8 | 2557 | -1.8% | 0.09 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 99.5 | 10.4 | 98.9 | 5.69 | 8.39 | 2.30 | -0.6% | 0.07 | BELOW FLOOR |
| P vs M | q1_p99_us | 499.1 | 65.7 | 686.6 | 240.8 | 176.5 | 338.7 | +37.6% | 1.06 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 202.9 | 13.9 | 199.7 | 14.3 | 14.1 | 36.6 | -1.6% | 0.23 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 265.0 | 43.3 | 254.8 | 18.6 | 33.3 | 163.6 | -3.8% | 0.30 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 682.4 | 31.2 | 693.2 | 21.6 | 26.9 | 50.7 | +1.6% | 0.40 | BELOW FLOOR |
| P vs M | q5_p50_us | 156.1 | 19.2 | 148.7 | 20.0 | 19.6 | 32.5 | -4.8% | 0.38 | BELOW FLOOR |
| P vs M | q6_p50_us | 2678 | 156.8 | 2784 | 198.6 | 178.9 | 428.2 | +4.0% | 0.59 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 55.1 | 26.2 | 83.4 | 24.5 | 25.3 | 5.28 | +51.4% | 1.12 | no difference |
| P vs M | c2_q1_p99_us | 1518 | 307.8 | 1063 | 143.1 | 240.0 | 1456 | -30.0% | 1.90 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 164.1 | 40.0 | 194.4 | 48.8 | 44.6 | 22.9 | +18.5% | 0.68 | no difference |
| P vs M | c4_q1_p50_us | 111.6 | 44.6 | 133.3 | 36.0 | 40.5 | 96.9 | +19.5% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2297 | 220.6 | 2413 | 177.0 | 200.0 | 305.6 | +5.1% | 0.58 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 239.2 | 53.3 | 272.4 | 36.5 | 45.7 | 78.0 | +13.8% | 0.72 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2860 | 942.6 | 2982 | 679.7 | 821.7 | 642.2 | +4.3% | 0.15 | BELOW FLOOR |
| P vs M | write_p50_us | 115774 | 1789 | 115662 | 1590 | 1693 | 6503 | -0.1% | 0.07 | BELOW FLOOR |
| P vs M | write_p99_us | 153063 | 8309 | 148105 | 9742 | 9054 | 37813 | -3.2% | 0.55 | BELOW FLOOR |
| P vs M | commits_per_s | 8.37 | 0.14 | 8.53 | 0.16 | 0.15 | 0.54 | +2.0% | 1.09 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 421.3 | 84.9 | 421.3 | 86.0 | 85.5 | 1293 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.7 | 0.03 | 45.0 | 0.02 | 0.03 | 0.15 | +0.5% | 8.39 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -835.9 | 175.8 | -730.3 | 141.0 | 159.4 | 2535 | -12.6% | 0.66 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 188.0 | 19.7 | 312.6 | 27.3 | 23.8 | 7.58 | +66.2% | 5.23 | **N better** |
| N vs H3 | q1_p99_us | 627.7 | 90.6 | 1324 | 319.6 | 234.9 | 233.6 | +110.8% | 2.96 | no difference |
| N vs H3 | q2_p50_us | 145.8 | 10.7 | 332.5 | 20.3 | 16.3 | 13.0 | +128.0% | 11.5 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 187.6 | 30.6 | 314.9 | 70.9 | 54.6 | 135.5 | +67.8% | 2.33 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q4_p50_us | 19884 | 334.8 | 11612 | 658.1 | 522.1 | 2371 | -41.6% | 15.8 | **H3 better** |
| N vs H3 | q5_p50_us | 30710 | 771.9 | 36734 | 1560 | 1230 | 3985 | +19.6% | 4.90 | **N better** |
| N vs H3 | q6_p50_us | 48110 | 1332 | 39986 | 1754 | 1557 | 18935 | -16.9% | 5.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p50_us | 119.8 | 19.1 | 294.2 | 28.3 | 24.2 | 54.8 | +145.5% | 7.22 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 642.3 | 226.2 | 1596 | 298.0 | 264.5 | 796.8 | +148.6% | 3.61 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 101.7 | 11.3 | 292.6 | 45.7 | 33.3 | 38.3 | +187.6% | 5.73 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 238.8 | 45.8 | 509.4 | 56.9 | 51.6 | 161.6 | +113.4% | 5.24 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 1086 | 261.2 | 1796 | 179.0 | 223.9 | 3539 | +65.3% | 3.17 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 186.8 | 29.4 | 538.0 | 78.1 | 59.0 | 172.5 | +188.0% | 5.95 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 1286 | 226.3 | 2298 | 403.0 | 326.9 | 3258 | +78.6% | 3.09 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 560.9 | 24.1 | 1210 | 91.1 | 66.6 | 184.8 | +115.8% | 9.74 | **N better** |
| N vs H3 | write_p99_us | 1036 | 139.8 | 3534 | 333.9 | 256.0 | 3396 | +241.0% | 9.76 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | commits_per_s | 1573 | 154.3 | 698.2 | 66.3 | 118.8 | 301.5 | -55.6% | 7.36 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 744.8 | 135.1 | 117.3 | 2.12 | 95.6 | 1446 | -84.2% | 6.57 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 44.3 | 0.84 | 112.5 | 0.26 | 0.62 | 2.38 | +154.3% | 109.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 4986 | 433.4 | 21136 | 4242 | 3015 | 44307 | +323.9% | 5.36 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 621.0 | 31.0 | 312.6 | 27.3 | 29.2 | 45.4 | -49.7% | 10.6 | **H3 better** |
| P+ vs H3 | q1_p99_us | 1674 | 282.5 | 1324 | 319.6 | 301.6 | 1323 | -21.0% | 1.16 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 499.5 | 36.9 | 332.5 | 20.3 | 29.8 | 35.9 | -33.4% | 5.60 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 305.7 | 20.3 | 314.9 | 70.9 | 52.2 | 130.0 | +3.0% | 0.18 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q4_p50_us | 11528 | 556.2 | 11612 | 658.1 | 609.3 | 883.4 | +0.7% | 0.14 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 37835 | 1201 | 36734 | 1560 | 1392 | 3751 | -2.9% | 0.79 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 39852 | 1907 | 39986 | 1754 | 1832 | 12446 | +0.3% | 0.07 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p50_us | 688.2 | 79.6 | 294.2 | 28.3 | 59.7 | 74.9 | -57.2% | 6.60 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 6370 | 825.7 | 1596 | 298.0 | 620.7 | 1358 | -74.9% | 7.69 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 550.6 | 50.2 | 292.6 | 45.7 | 48.0 | 18.1 | -46.9% | 5.38 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1932 | 85.6 | 509.4 | 56.9 | 72.7 | 236.5 | -73.6% | 19.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 8880 | 598.6 | 1796 | 179.0 | 441.8 | 3628 | -79.8% | 16.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1653 | 88.7 | 538.0 | 78.1 | 83.6 | 186.1 | -67.5% | 13.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 6442 | 856.5 | 2298 | 403.0 | 669.3 | 3132 | -64.3% | 6.19 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 847.1 | 38.6 | 1210 | 91.1 | 70.0 | 192.2 | +42.9% | 5.19 | **P+ better** |
| P+ vs H3 | write_p99_us | 2812 | 368.4 | 3534 | 333.9 | 351.6 | 3376 | +25.7% | 2.05 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 973.4 | 73.4 | 698.2 | 66.3 | 70.0 | 117.1 | -28.3% | 3.93 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 852.0 | 155.0 | 117.3 | 2.12 | 109.6 | 1416 | -86.2% | 6.70 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.7 | 0.02 | 112.5 | 0.26 | 0.19 | 0.32 | +151.8% | 364.9 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -850.8 | 167.6 | 21136 | 4242 | 3002 | 42229 | -2584.2% | 7.32 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 177.5 | 20.9 | 614.1 | 40.1 | 32.0 | 126.6 | +246.1% | 13.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 599.5 | 62.3 | 2103 | 599.8 | 426.4 | 1159 | +250.9% | 3.53 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 150.7 | 19.6 | 471.3 | 51.8 | 39.1 | 120.5 | +212.7% | 8.19 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 164.5 | 17.5 | 275.4 | 28.9 | 23.9 | 23.3 | +67.5% | 4.64 | **N better** |
| N vs P+ | q4_p50_us | 20015 | 376.3 | 11180 | 856.0 | 661.2 | 1610 | -44.1% | 13.4 | **P+ better** |
| N vs P+ | q5_p50_us | 30314 | 868.0 | 36790 | 1111 | 996.8 | 1862 | +21.4% | 6.50 | **N better** |
| N vs P+ | q6_p50_us | 47659 | 816.7 | 38022 | 1398 | 1145 | 8520 | -20.2% | 8.42 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 117.5 | 18.7 | 685.5 | 52.5 | 39.4 | 83.4 | +483.5% | 14.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 454.1 | 106.3 | 4304 | 921.3 | 655.8 | 4780 | +847.8% | 5.87 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 87.9 | 11.3 | 529.8 | 75.5 | 54.0 | 111.4 | +502.7% | 8.19 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 226.2 | 14.1 | 1823 | 55.5 | 40.5 | 730.9 | +706.0% | 39.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p99_us | 884.1 | 87.8 | 7845 | 1722 | 1219 | 2282 | +787.3% | 5.71 | **N better** |
| N vs P+ | c4_q2_p50_us | 166.8 | 18.5 | 1574 | 121.1 | 86.6 | 259.3 | +843.8% | 16.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1207 | 400.9 | 7350 | 1565 | 1143 | 5552 | +508.9% | 5.38 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 564.3 | 43.2 | 825.7 | 75.9 | 61.8 | 68.7 | +46.3% | 4.23 | **N better** |
| N vs P+ | write_p99_us | 939.4 | 184.1 | 2676 | 333.4 | 269.3 | 900.6 | +184.9% | 6.45 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 1633 | 201.7 | 1042 | 73.2 | 151.8 | 531.1 | -36.2% | 3.89 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 737.6 | 128.0 | 855.4 | 141.3 | 134.8 | 1980 | +16.0% | 0.87 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 41.9 | 0.55 | 44.7 | 0.01 | 0.39 | 7.82 | +6.8% | 7.36 | BELOW FLOOR |
| N vs P+ | pss_growth_bytes_per_key_read | 4754 | 901.5 | -809.6 | 158.7 | 647.3 | 2885 | -117.0% | 8.59 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 177.5 | 20.9 | 104.6 | 7.52 | 15.7 | 40.9 | -41.1% | 4.64 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q1_p99_us | 599.5 | 62.3 | 543.8 | 77.7 | 70.4 | 259.6 | -9.3% | 0.79 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 150.7 | 19.6 | 205.2 | 12.4 | 16.4 | 17.6 | +36.1% | 3.32 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 164.5 | 17.5 | 245.2 | 30.0 | 24.5 | 139.0 | +49.1% | 3.29 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q4_p50_us | 20015 | 376.3 | 707.3 | 38.1 | 267.5 | 394.6 | -96.5% | 72.2 | **P better** |
| N vs P | q5_p50_us | 30314 | 868.0 | 173.0 | 29.0 | 614.1 | 1325 | -99.4% | 49.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 47659 | 816.7 | 2671 | 81.8 | 580.4 | 6488 | -94.4% | 77.5 | **P better** |
| N vs P | c2_q1_p50_us | 117.5 | 18.7 | 82.1 | 16.0 | 17.4 | 42.3 | -30.1% | 2.03 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 454.1 | 106.3 | 1706 | 383.3 | 281.2 | 467.4 | +275.7% | 4.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 87.9 | 11.3 | 217.4 | 49.5 | 35.9 | 93.0 | +147.3% | 3.60 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 226.2 | 14.1 | 119.2 | 26.4 | 21.2 | 38.8 | -47.3% | 5.05 | **P better** |
| N vs P | c4_q1_p99_us | 884.1 | 87.8 | 2306 | 262.6 | 195.8 | 1122 | +160.8% | 7.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 166.8 | 18.5 | 303.3 | 54.2 | 40.5 | 91.6 | +81.9% | 3.37 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 1207 | 400.9 | 2597 | 375.6 | 388.4 | 914.8 | +115.1% | 3.58 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 564.3 | 43.2 | 117021 | 2855 | 2019 | 875.1 | +20636.0% | 57.7 | **N better** |
| N vs P | write_p99_us | 939.4 | 184.1 | 147782 | 12732 | 9004 | 3121 | +15631.9% | 16.3 | **N better** |
| N vs P | commits_per_s | 1633 | 201.7 | 8.23 | 0.26 | 142.7 | 504.2 | -99.5% | 11.4 | **N better** |
| N vs P | view_state_bytes_per_key_read | 737.6 | 128.0 | 418.2 | 79.9 | 106.7 | 1670 | -43.3% | 2.99 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 41.9 | 0.55 | 44.8 | 0.02 | 0.39 | 7.82 | +6.9% | 7.44 | BELOW FLOOR |
| N vs P | pss_growth_bytes_per_key_read | 4754 | 901.5 | -758.0 | 146.3 | 645.8 | 2986 | -115.9% | 8.53 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 614.1 | 40.1 | 104.6 | 7.52 | 28.9 | 119.9 | -83.0% | 17.7 | **P better** |
| P+ vs P | q1_p99_us | 2103 | 599.8 | 543.8 | 77.7 | 427.7 | 1153 | -74.1% | 3.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 471.3 | 51.8 | 205.2 | 12.4 | 37.6 | 121.2 | -56.5% | 7.07 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 275.4 | 28.9 | 245.2 | 30.0 | 29.5 | 140.2 | -10.9% | 1.02 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 11180 | 856.0 | 707.3 | 38.1 | 605.9 | 1566 | -93.7% | 17.3 | **P better** |
| P+ vs P | q5_p50_us | 36790 | 1111 | 173.0 | 29.0 | 785.8 | 1311 | -99.5% | 46.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 38022 | 1398 | 2671 | 81.8 | 990.3 | 5523 | -93.0% | 35.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 685.5 | 52.5 | 82.1 | 16.0 | 38.8 | 72.3 | -88.0% | 15.5 | **P better** |
| P+ vs P | c2_q1_p99_us | 4304 | 921.3 | 1706 | 383.3 | 705.6 | 4789 | -60.4% | 3.68 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 529.8 | 75.5 | 217.4 | 49.5 | 63.8 | 117.6 | -59.0% | 4.89 | **P better** |
| P+ vs P | c4_q1_p50_us | 1823 | 55.5 | 119.2 | 26.4 | 43.4 | 730.8 | -93.5% | 39.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p99_us | 7845 | 1722 | 2306 | 262.6 | 1232 | 2376 | -70.6% | 4.50 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1574 | 121.1 | 303.3 | 54.2 | 93.8 | 244.6 | -80.7% | 13.5 | **P better** |
| P+ vs P | c4_q2_p99_us | 7350 | 1565 | 2597 | 375.6 | 1138 | 5504 | -64.7% | 4.18 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 825.7 | 75.9 | 117021 | 2855 | 2019 | 872.4 | +14071.6% | 57.5 | **P+ better** |
| P+ vs P | write_p99_us | 2676 | 333.4 | 147782 | 12732 | 9006 | 3242 | +5422.9% | 16.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 1042 | 73.2 | 8.23 | 0.26 | 51.8 | 166.6 | -99.2% | 20.0 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 855.4 | 141.3 | 418.2 | 79.9 | 114.8 | 1646 | -51.1% | 3.81 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 44.7 | 0.01 | 44.8 | 0.02 | 0.02 | 0.19 | +0.1% | 1.85 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -809.6 | 158.7 | -758.0 | 146.3 | 152.6 | 2396 | -6.4% | 0.34 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 104.6 | 7.52 | 101.2 | 4.55 | 6.22 | 24.9 | -3.2% | 0.53 | BELOW FLOOR |
| P vs M | q1_p99_us | 543.8 | 77.7 | 672.8 | 245.8 | 182.3 | 179.5 | +23.7% | 0.71 | BELOW FLOOR |
| P vs M | q2_p50_us | 205.2 | 12.4 | 209.4 | 16.4 | 14.5 | 24.0 | +2.0% | 0.29 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 245.2 | 30.0 | 245.8 | 38.9 | 34.7 | 141.8 | +0.2% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q4_p50_us | 707.3 | 38.1 | 722.8 | 22.7 | 31.3 | 92.4 | +2.2% | 0.50 | BELOW FLOOR |
| P vs M | q5_p50_us | 173.0 | 29.0 | 162.8 | 23.0 | 26.2 | 55.6 | -5.9% | 0.39 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 2671 | 81.8 | 2679 | 72.8 | 77.4 | 977.7 | +0.3% | 0.11 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 82.1 | 16.0 | 34.1 | 5.87 | 12.1 | 4.85 | -58.4% | 3.97 | **M better** |
| P vs M | c2_q1_p99_us | 1706 | 383.3 | 1196 | 221.0 | 312.9 | 1555 | -29.9% | 1.63 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 217.4 | 49.5 | 125.2 | 17.1 | 37.0 | 72.1 | -42.4% | 2.49 | no difference |
| P vs M | c4_q1_p50_us | 119.2 | 26.4 | 111.4 | 18.3 | 22.7 | 69.3 | -6.5% | 0.34 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2306 | 262.6 | 2132 | 311.1 | 287.9 | 985.1 | -7.5% | 0.60 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 303.3 | 54.2 | 261.2 | 41.6 | 48.3 | 152.6 | -13.9% | 0.87 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p99_us | 2597 | 375.6 | 2978 | 873.7 | 672.5 | 553.9 | +14.7% | 0.57 | BELOW FLOOR |
| P vs M | write_p50_us | 117021 | 2855 | 117072 | 589.0 | 2061 | 2174 | +0.0% | 0.02 | BELOW FLOOR |
| P vs M | write_p99_us | 147782 | 12732 | 154357 | 13126 | 12930 | 26343 | +4.4% | 0.51 | BELOW FLOOR |
| P vs M | commits_per_s | 8.23 | 0.26 | 8.41 | 0.07 | 0.19 | 0.77 | +2.2% | 0.96 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 418.2 | 79.9 | 416.8 | 77.5 | 78.7 | 1257 | -0.3% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 44.8 | 0.02 | 44.9 | 0.03 | 0.02 | 0.23 | +0.4% | 6.69 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -758.0 | 146.3 | -675.2 | 142.9 | 144.6 | 2248 | -10.9% | 0.57 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 177.5 | 20.9 | 333.9 | 44.6 | 34.8 | 41.7 | +88.1% | 4.49 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 599.5 | 62.3 | 1718 | 250.7 | 182.6 | 204.2 | +186.5% | 6.12 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 150.7 | 19.6 | 318.1 | 31.2 | 26.0 | 82.5 | +111.0% | 6.43 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 164.5 | 17.5 | 316.8 | 38.1 | 29.6 | 110.4 | +92.6% | 5.14 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q4_p50_us | 20015 | 376.3 | 11561 | 615.3 | 510.0 | 3716 | -42.2% | 16.6 | **H3 better** |
| N vs H3 | q5_p50_us | 30314 | 868.0 | 37804 | 2123 | 1622 | 3661 | +24.7% | 4.62 | **N better** |
| N vs H3 | q6_p50_us | 47659 | 816.7 | 40968 | 939.3 | 880.1 | 9861 | -14.0% | 7.60 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 117.5 | 18.7 | 315.6 | 34.5 | 27.8 | 93.7 | +168.6% | 7.14 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 454.1 | 106.3 | 1467 | 373.8 | 274.8 | 721.9 | +222.9% | 3.68 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 87.9 | 11.3 | 341.7 | 42.3 | 30.9 | 60.9 | +288.8% | 8.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 226.2 | 14.1 | 556.8 | 49.6 | 36.5 | 123.2 | +146.2% | 9.06 | **N better** |
| N vs H3 | c4_q1_p99_us | 884.1 | 87.8 | 2072 | 273.7 | 203.3 | 657.6 | +134.3% | 5.84 | **N better** |
| N vs H3 | c4_q2_p50_us | 166.8 | 18.5 | 535.0 | 52.8 | 39.5 | 91.1 | +220.9% | 9.32 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1207 | 400.9 | 2016 | 295.7 | 352.2 | 965.4 | +67.0% | 2.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 564.3 | 43.2 | 1232 | 68.6 | 57.3 | 96.7 | +118.3% | 11.6 | **N better** |
| N vs H3 | write_p99_us | 939.4 | 184.1 | 4253 | 478.3 | 362.4 | 247.0 | +352.8% | 9.14 | **N better** |
| N vs H3 | commits_per_s | 1633 | 201.7 | 668.1 | 48.8 | 146.8 | 510.3 | -59.1% | 6.57 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 737.6 | 128.0 | 119.1 | 3.35 | 90.5 | 1415 | -83.9% | 6.83 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 41.9 | 0.55 | 116.4 | 0.16 | 0.40 | 8.20 | +177.9% | 184.2 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 4754 | 901.5 | 22262 | 4340 | 3135 | 35881 | +368.3% | 5.59 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 614.1 | 40.1 | 333.9 | 44.6 | 42.4 | 120.2 | -45.6% | 6.61 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2103 | 599.8 | 1718 | 250.7 | 459.7 | 1142 | -18.3% | 0.84 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 471.3 | 51.8 | 318.1 | 31.2 | 42.8 | 145.6 | -32.5% | 3.58 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 275.4 | 28.9 | 316.8 | 38.1 | 33.8 | 111.9 | +15.0% | 1.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 11180 | 856.0 | 11561 | 615.3 | 745.4 | 4013 | +3.4% | 0.51 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 36790 | 1111 | 37804 | 2123 | 1694 | 3656 | +2.8% | 0.60 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 38022 | 1398 | 40968 | 939.3 | 1191 | 9254 | +7.7% | 2.47 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 685.5 | 52.5 | 315.6 | 34.5 | 44.4 | 110.5 | -54.0% | 8.32 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4304 | 921.3 | 1467 | 373.8 | 703.0 | 4820 | -65.9% | 4.04 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 529.8 | 75.5 | 341.7 | 42.3 | 61.1 | 94.3 | -35.5% | 3.08 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1823 | 55.5 | 556.8 | 49.6 | 52.6 | 740.1 | -69.5% | 24.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p99_us | 7845 | 1722 | 2072 | 273.7 | 1233 | 2196 | -73.6% | 4.68 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1574 | 121.1 | 535.0 | 52.8 | 93.4 | 244.4 | -66.0% | 11.1 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7350 | 1565 | 2016 | 295.7 | 1126 | 5512 | -72.6% | 4.74 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 825.7 | 75.9 | 1232 | 68.6 | 72.3 | 68.3 | +49.2% | 5.62 | **P+ better** |
| P+ vs H3 | write_p99_us | 2676 | 333.4 | 4253 | 478.3 | 412.3 | 911.4 | +59.0% | 3.83 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 1042 | 73.2 | 668.1 | 48.8 | 62.2 | 184.1 | -35.9% | 6.01 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 855.4 | 141.3 | 119.1 | 3.35 | 100.0 | 1386 | -86.1% | 7.37 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 44.7 | 0.01 | 116.4 | 0.16 | 0.11 | 2.49 | +160.1% | 627.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -809.6 | 158.7 | 22262 | 4340 | 3071 | 35837 | -2849.8% | 7.51 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 141.9 | 11.7 | 575.5 | 24.1 | 18.9 | 191.7 | +305.5% | 22.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q1_p99_us | 1199 | 507.3 | 3194 | 622.1 | 567.6 | 695.0 | +166.3% | 3.51 | **N better** |
| N vs P+ | q2_p50_us | 193.4 | 8.39 | 490.7 | 41.5 | 29.9 | 61.9 | +153.7% | 9.94 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 230.3 | 17.1 | 293.5 | 24.9 | 21.4 | 105.7 | +27.4% | 2.96 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | 196633 | 7357 | 71505 | 4034 | 5933 | 5871 | -63.6% | 21.1 | **P+ better** |
| N vs P+ | q5_p50_us | 628163 | 11097 | 805437 | 25590 | 19723 | 53529 | +28.2% | 8.99 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 62.2 | 14.8 | 548.9 | 49.1 | 36.2 | 33.1 | +782.6% | 13.4 | **N better** |
| N vs P+ | c2_q1_p99_us | 341.6 | 76.4 | 4938 | 660.5 | 470.1 | 3044 | +1345.4% | 9.78 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 77.1 | 20.8 | 475.1 | 49.2 | 37.8 | 4.08 | +516.6% | 10.5 | **N better** |
| N vs P+ | c4_q1_p50_us | 101.0 | 8.15 | 1560 | 201.6 | 142.7 | 445.9 | +1444.4% | 10.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 965.4 | 377.8 | 6541 | 943.7 | 718.8 | 5609 | +577.5% | 7.76 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 129.5 | 14.4 | 1438 | 119.4 | 85.0 | 106.5 | +1010.2% | 15.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 2046 | 944.6 | 6283 | 1838 | 1461 | 7131 | +207.2% | 2.90 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 645.9 | 83.1 | 1072 | 95.1 | 89.3 | 341.5 | +65.9% | 4.77 | **N better** |
| N vs P+ | write_p99_us | 2280 | 891.0 | 3205 | 126.8 | 636.4 | 7117 | +40.6% | 1.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1253 | 158.5 | 772.0 | 60.9 | 120.1 | 347.2 | -38.4% | 4.00 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 4071 | 973.5 | 4406 | 1072 | 1024 | 15946 | +8.2% | 0.33 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 314.4 | 9.82 | 494.6 | 0.13 | 6.95 | 5.20 | +57.3% | 25.9 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 15404 | 2112 | 47381 | 11683 | 8395 | 121703 | +207.6% | 3.81 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 141.9 | 11.7 | 106.1 | 1.82 | 8.38 | 44.8 | -25.3% | 4.28 | BELOW FLOOR |
| N vs P | q1_p99_us | 1199 | 507.3 | 1067 | 499.1 | 503.3 | 293.4 | -11.0% | 0.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 193.4 | 8.39 | 215.0 | 19.5 | 15.0 | 74.0 | +11.2% | 1.43 | BELOW FLOOR |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 230.3 | 17.1 | 254.4 | 30.6 | 24.8 | 154.9 | +10.5% | 0.97 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q4_p50_us | 196633 | 7357 | 4908 | 92.7 | 5202 | 264.0 | -97.5% | 36.9 | **P better** |
| N vs P | q5_p50_us | 628163 | 11097 | 171.4 | 10.7 | 7847 | 44475 | -100.0% | 80.0 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 62.2 | 14.8 | 35.1 | 6.48 | 11.4 | 3.35 | -43.6% | 2.37 | no difference |
| N vs P | c2_q1_p99_us | 341.6 | 76.4 | 1319 | 358.8 | 259.4 | 401.5 | +286.0% | 3.77 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 77.1 | 20.8 | 144.0 | 23.0 | 22.0 | 29.6 | +86.9% | 3.05 | **N better** |
| N vs P | c4_q1_p50_us | 101.0 | 8.15 | 98.4 | 26.3 | 19.5 | 113.6 | -2.6% | 0.13 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 965.4 | 377.8 | 2602 | 496.8 | 441.3 | 591.1 | +169.5% | 3.71 | **N better** |
| N vs P | c4_q2_p50_us | 129.5 | 14.4 | 241.7 | 67.3 | 48.7 | 159.4 | +86.6% | 2.30 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 2046 | 944.6 | 6293 | 2813 | 2099 | 12201 | +207.7% | 2.02 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 645.9 | 83.1 | 1180417 | 8859 | 6264 | 15648 | +182644.7% | 188.3 | **N better** |
| N vs P | write_p99_us | 2280 | 891.0 | 1305100 | 20462 | 14483 | 7070 | +57149.8% | 90.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1253 | 158.5 | 0.84 | 0.02 | 112.1 | 333.9 | -99.9% | 11.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | view_state_bytes_per_key_read | 4071 | 973.5 | 2881 | 712.2 | 852.9 | 13381 | -29.2% | 1.40 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 314.4 | 9.82 | 488.9 | 4.70 | 7.70 | 1.16 | +55.5% | 22.7 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 15404 | 2112 | 53517 | 12946 | 9276 | 135085 | +247.4% | 4.11 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 575.5 | 24.1 | 106.1 | 1.82 | 17.1 | 195.3 | -81.6% | 27.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q1_p99_us | 3194 | 622.1 | 1067 | 499.1 | 564.0 | 695.3 | -66.6% | 3.77 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 490.7 | 41.5 | 215.0 | 19.5 | 32.4 | 60.2 | -56.2% | 8.51 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 293.5 | 24.9 | 254.4 | 30.6 | 27.9 | 132.3 | -13.3% | 1.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 71505 | 4034 | 4908 | 92.7 | 2853 | 5865 | -93.1% | 23.3 | **P better** |
| P+ vs P | q5_p50_us | 805437 | 25590 | 171.4 | 10.7 | 18095 | 29788 | -100.0% | 44.5 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 548.9 | 49.1 | 35.1 | 6.48 | 35.0 | 33.3 | -93.6% | 14.7 | **P better** |
| P+ vs P | c2_q1_p99_us | 4938 | 660.5 | 1319 | 358.8 | 531.5 | 3068 | -73.3% | 6.81 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 475.1 | 49.2 | 144.0 | 23.0 | 38.4 | 29.5 | -69.7% | 8.61 | **P better** |
| P+ vs P | c4_q1_p50_us | 1560 | 201.6 | 98.4 | 26.3 | 143.8 | 432.7 | -93.7% | 10.2 | **P better** |
| P+ vs P | c4_q1_p99_us | 6541 | 943.7 | 2602 | 496.8 | 754.1 | 5622 | -60.2% | 5.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1438 | 119.4 | 241.7 | 67.3 | 96.9 | 122.1 | -83.2% | 12.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 6283 | 1838 | 6293 | 2813 | 2376 | 13374 | +0.2% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 1072 | 95.1 | 1180417 | 8859 | 6265 | 15648 | +110045.9% | 188.3 | **P+ better** |
| P+ vs P | write_p99_us | 3205 | 126.8 | 1305100 | 20462 | 14469 | 815.9 | +40615.0% | 90.0 | **P+ better** |
| P+ vs P | commits_per_s | 772.0 | 60.9 | 0.84 | 0.02 | 43.0 | 95.2 | -99.9% | 17.9 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 4406 | 1072 | 2881 | 712.2 | 910.2 | 14134 | -34.6% | 1.68 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 494.6 | 0.13 | 488.9 | 4.70 | 3.32 | 5.19 | -1.2% | 1.71 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 47381 | 11683 | 53517 | 12946 | 12331 | 176275 | +13.0% | 0.50 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 106.1 | 1.82 | 107.6 | 5.41 | 4.04 | 41.4 | +1.5% | 0.39 | BELOW FLOOR |
| P vs M | q1_p99_us | 1067 | 499.1 | 1176 | 588.3 | 545.5 | 208.4 | +10.2% | 0.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 215.0 | 19.5 | 218.2 | 28.4 | 24.4 | 68.5 | +1.5% | 0.13 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 254.4 | 30.6 | 249.9 | 41.5 | 36.5 | 158.6 | -1.8% | 0.12 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 4908 | 92.7 | 4945 | 138.7 | 118.0 | 77.7 | +0.7% | 0.31 | BELOW FLOOR |
| P vs M | q5_p50_us | 171.4 | 10.7 | 150.9 | 19.2 | 15.5 | 39.8 | -12.0% | 1.32 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 35.1 | 6.48 | 33.6 | 2.70 | 4.96 | 39.4 | -4.3% | 0.31 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1319 | 358.8 | 1174 | 235.1 | 303.3 | 756.6 | -11.0% | 0.48 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 144.0 | 23.0 | 149.5 | 25.2 | 24.1 | 48.5 | +3.8% | 0.23 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 98.4 | 26.3 | 122.0 | 16.3 | 21.9 | 30.7 | +24.0% | 1.08 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2602 | 496.8 | 2860 | 515.9 | 506.4 | 735.3 | +9.9% | 0.51 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 241.7 | 67.3 | 293.4 | 63.7 | 65.5 | 126.4 | +21.4% | 0.79 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 6293 | 2813 | 6829 | 2893 | 2854 | 11810 | +8.5% | 0.19 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 1180417 | 8859 | 1188041 | 7615 | 8260 | 15834 | +0.6% | 0.92 | BELOW FLOOR |
| P vs M | write_p99_us | 1305100 | 20462 | 1338214 | 31791 | 26734 | 26788 | +2.5% | 1.24 | no difference |
| P vs M | commits_per_s | 0.84 | 0.02 | 0.84 | 0.01 | 0.01 | 0.01 | -0.9% | 0.67 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 2881 | 712.2 | 2881 | 712.2 | 712.2 | 11159 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 488.9 | 4.70 | 489.1 | 0.61 | 3.35 | 0.98 | +0.0% | 0.06 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 53517 | 12946 | 53286 | 13062 | 13004 | 185220 | -0.4% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 141.9 | 11.7 | 265.7 | 17.8 | 15.1 | 24.3 | +87.2% | 8.20 | **N better** |
| N vs H3 | q1_p99_us | 1199 | 507.3 | 2456 | 1092 | 851.6 | 2339 | +104.7% | 1.48 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 193.4 | 8.39 | 267.4 | 46.0 | 33.1 | 55.8 | +38.3% | 2.24 | no difference |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 230.3 | 17.1 | 331.5 | 59.8 | 44.0 | 106.2 | +43.9% | 2.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | 196633 | 7357 | 71167 | 2573 | 5511 | 1600 | -63.8% | 22.8 | **H3 better** |
| N vs H3 | q5_p50_us | 628163 | 11097 | 786076 | 15032 | 13212 | 45261 | +25.1% | 12.0 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 62.2 | 14.8 | 280.2 | 68.0 | 49.2 | 88.8 | +350.5% | 4.43 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 341.6 | 76.4 | 3142 | 1011 | 717.1 | 6087 | +819.8% | 3.91 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 77.1 | 20.8 | 259.5 | 78.0 | 57.1 | 114.3 | +236.7% | 3.19 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 101.0 | 8.15 | 253.3 | 57.6 | 41.1 | 113.4 | +150.7% | 3.70 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 965.4 | 377.8 | 4897 | 722.4 | 576.5 | 515.2 | +407.3% | 6.82 | **N better** |
| N vs H3 | c4_q2_p50_us | 129.5 | 14.4 | 299.2 | 80.0 | 57.5 | 116.9 | +131.0% | 2.95 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 2046 | 944.6 | 4718 | 880.6 | 913.2 | 3734 | +130.6% | 2.93 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 645.9 | 83.1 | 1536 | 120.4 | 103.5 | 267.6 | +137.9% | 8.61 | **N better** |
| N vs H3 | write_p99_us | 2280 | 891.0 | 4458 | 421.2 | 696.9 | 7313 | +95.6% | 3.13 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1253 | 158.5 | 536.2 | 30.1 | 114.1 | 376.7 | -57.2% | 6.28 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | view_state_bytes_per_key_read | 4071 | 973.5 | 193.8 | 2.21 | 688.4 | 10806 | -95.2% | 5.63 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 314.4 | 9.82 | 535.7 | 0.66 | 6.96 | 0.83 | +70.4% | 31.8 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 15404 | 2112 | 65934 | 12517 | 8976 | 181071 | +328.0% | 5.63 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 575.5 | 24.1 | 265.7 | 17.8 | 21.2 | 191.6 | -53.8% | 14.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q1_p99_us | 3194 | 622.1 | 2456 | 1092 | 888.9 | 2422 | -23.1% | 0.83 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 490.7 | 41.5 | 267.4 | 46.0 | 43.8 | 35.6 | -45.5% | 5.10 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 293.5 | 24.9 | 331.5 | 59.8 | 45.8 | 69.4 | +12.9% | 0.83 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 71505 | 4034 | 71167 | 2573 | 3383 | 6073 | -0.5% | 0.10 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 805437 | 25590 | 786076 | 15032 | 20986 | 30949 | -2.4% | 0.92 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 548.9 | 49.1 | 280.2 | 68.0 | 59.3 | 94.7 | -49.0% | 4.53 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 4938 | 660.5 | 3142 | 1011 | 854.1 | 6805 | -36.4% | 2.10 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 475.1 | 49.2 | 259.5 | 78.0 | 65.2 | 114.3 | -45.4% | 3.31 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1560 | 201.6 | 253.3 | 57.6 | 148.3 | 432.7 | -83.8% | 8.82 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6541 | 943.7 | 4897 | 722.4 | 840.4 | 5614 | -25.1% | 1.96 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1438 | 119.4 | 299.2 | 80.0 | 101.6 | 56.3 | -79.2% | 11.2 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 6283 | 1838 | 4718 | 880.6 | 1441 | 6629 | -24.9% | 1.09 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1072 | 95.1 | 1536 | 120.4 | 108.5 | 263.0 | +43.4% | 4.28 | **P+ better** |
| P+ vs H3 | write_p99_us | 3205 | 126.8 | 4458 | 421.2 | 311.0 | 2040 | +39.1% | 4.03 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 772.0 | 60.9 | 536.2 | 30.1 | 48.0 | 198.6 | -30.5% | 4.91 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | view_state_bytes_per_key_read | 4406 | 1072 | 193.8 | 2.21 | 758.2 | 11727 | -95.6% | 5.56 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 494.6 | 0.13 | 535.7 | 0.66 | 0.47 | 5.13 | +8.3% | 86.6 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 47381 | 11683 | 65934 | 12517 | 12107 | 213569 | +39.2% | 1.53 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 121.1 | 6.17 | 722.6 | 28.2 | 20.4 | 131.0 | +496.8% | 29.5 | **N better** |
| N vs P+ | q1_p99_us | 1292 | 284.9 | 3669 | 417.7 | 357.5 | 1355 | +184.1% | 6.65 | **N better** |
| N vs P+ | q2_p50_us | 151.4 | 10.1 | 627.0 | 64.3 | 46.1 | 105.5 | +314.2% | 10.3 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 166.4 | 32.7 | 290.9 | 45.1 | 39.4 | 85.6 | +74.8% | 3.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q5_p50_us | 621480 | 11217 | 820951 | 12224 | 11731 | 23525 | +32.1% | 17.0 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 78.6 | 24.0 | 671.1 | 87.0 | 63.8 | 26.1 | +754.0% | 9.28 | **N better** |
| N vs P+ | c2_q1_p99_us | 337.7 | 32.9 | 4510 | 276.2 | 196.7 | 404.3 | +1235.7% | 21.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 97.7 | 28.5 | 635.5 | 95.6 | 70.5 | 169.5 | +550.5% | 7.62 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 104.1 | 27.1 | 1674 | 157.0 | 112.7 | 114.8 | +1508.7% | 13.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 1102 | 523.6 | 6616 | 546.9 | 535.4 | 2250 | +500.5% | 10.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 133.7 | 27.9 | 1556 | 59.6 | 46.5 | 90.5 | +1063.7% | 30.6 | **N better** |
| N vs P+ | c4_q2_p99_us | 2805 | 1631 | 7470 | 1123 | 1400 | 3749 | +166.3% | 3.33 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 662.4 | 110.3 | 1174 | 63.2 | 89.9 | 106.4 | +77.2% | 5.69 | **N better** |
| N vs P+ | write_p99_us | 2218 | 387.5 | 3533 | 423.9 | 406.1 | 746.1 | +59.3% | 3.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1216 | 205.9 | 709.1 | 28.1 | 146.9 | 562.2 | -41.7% | 3.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 4038 | 982.7 | 4343 | 1070 | 1027 | 15610 | +7.6% | 0.30 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 354.4 | 12.2 | 492.2 | 0.49 | 8.62 | 26.1 | +38.9% | 16.0 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 26218 | 3614 | 50866 | 12727 | 9355 | 129062 | +94.0% | 2.63 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 121.1 | 6.17 | 107.1 | 7.54 | 6.89 | 47.5 | -11.6% | 2.03 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q1_p99_us | 1292 | 284.9 | 835.9 | 230.3 | 259.0 | 487.9 | -35.3% | 1.76 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 151.4 | 10.1 | 218.4 | 25.4 | 19.3 | 23.1 | +44.3% | 3.47 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 166.4 | 32.7 | 220.7 | 36.2 | 34.5 | 241.2 | +32.6% | 1.57 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q5_p50_us | 621480 | 11217 | 173.6 | 39.2 | 7932 | 1208 | -100.0% | 78.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 78.6 | 24.0 | 34.1 | 2.32 | 17.1 | 20.7 | -56.6% | 2.61 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 337.7 | 32.9 | 1027 | 64.1 | 50.9 | 264.5 | +204.1% | 13.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 97.7 | 28.5 | 127.8 | 16.8 | 23.4 | 80.7 | +30.8% | 1.29 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 104.1 | 27.1 | 83.4 | 10.8 | 20.6 | 50.1 | -19.8% | 1.00 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 1102 | 523.6 | 2662 | 349.9 | 445.3 | 764.1 | +141.6% | 3.50 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 133.7 | 27.9 | 235.0 | 34.9 | 31.6 | 39.9 | +75.8% | 3.21 | **N better** |
| N vs P | c4_q2_p99_us | 2805 | 1631 | 12147 | 4806 | 3589 | 715.9 | +333.1% | 2.60 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 662.4 | 110.3 | 1196826 | 12337 | 8724 | 8484 | +180568.1% | 137.1 | **N better** |
| N vs P | write_p99_us | 2218 | 387.5 | 1321551 | 43105 | 30481 | 10480 | +59483.0% | 43.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1216 | 205.9 | 0.83 | 0.01 | 145.6 | 559.8 | -99.9% | 8.35 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | view_state_bytes_per_key_read | 4038 | 982.7 | 2857 | 718.1 | 860.6 | 13108 | -29.3% | 1.37 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 354.4 | 12.2 | 486.6 | 0.83 | 8.63 | 26.1 | +37.3% | 15.3 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 26218 | 3614 | 52890 | 12899 | 9472 | 133725 | +101.7% | 2.82 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 722.6 | 28.2 | 107.1 | 7.54 | 20.6 | 139.1 | -85.2% | 29.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p99_us | 3669 | 417.7 | 835.9 | 230.3 | 337.2 | 1437 | -77.2% | 8.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 627.0 | 64.3 | 218.4 | 25.4 | 48.9 | 107.9 | -65.2% | 8.36 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 290.9 | 45.1 | 220.7 | 36.2 | 40.9 | 236.5 | -24.1% | 1.72 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q5_p50_us | 820951 | 12224 | 173.6 | 39.2 | 8644 | 23495 | -100.0% | 95.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 671.1 | 87.0 | 34.1 | 2.32 | 61.6 | 30.5 | -94.9% | 10.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 4510 | 276.2 | 1027 | 64.1 | 200.5 | 391.8 | -77.2% | 17.4 | **P better** |
| P+ vs P | c2_q2_p50_us | 635.5 | 95.6 | 127.8 | 16.8 | 68.6 | 170.7 | -79.9% | 7.40 | **P better** |
| P+ vs P | c4_q1_p50_us | 1674 | 157.0 | 83.4 | 10.8 | 111.3 | 125.2 | -95.0% | 14.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 6616 | 546.9 | 2662 | 349.9 | 459.1 | 2346 | -59.8% | 8.61 | **P better** |
| P+ vs P | c4_q2_p50_us | 1556 | 59.6 | 235.0 | 34.9 | 48.8 | 93.3 | -84.9% | 27.1 | **P better** |
| P+ vs P | c4_q2_p99_us | 7470 | 1123 | 12147 | 4806 | 3490 | 3772 | +62.6% | 1.34 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 1174 | 63.2 | 1196826 | 12337 | 8724 | 8484 | +101834.5% | 137.1 | **P+ better** |
| P+ vs P | write_p99_us | 3533 | 423.9 | 1321551 | 43105 | 30482 | 10455 | +37310.3% | 43.2 | **P+ better** |
| P+ vs P | commits_per_s | 709.1 | 28.1 | 0.83 | 0.01 | 19.9 | 51.1 | -99.9% | 35.7 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 4343 | 1070 | 2857 | 718.1 | 911.5 | 13834 | -34.2% | 1.63 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 492.2 | 0.49 | 486.6 | 0.83 | 0.68 | 0.90 | -1.1% | 8.23 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 50866 | 12727 | 52890 | 12899 | 12813 | 185825 | +4.0% | 0.16 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 107.1 | 7.54 | 105.5 | 5.90 | 6.77 | 64.7 | -1.5% | 0.23 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q1_p99_us | 835.9 | 230.3 | 1091 | 534.7 | 411.6 | 720.2 | +30.5% | 0.62 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 218.4 | 25.4 | 203.3 | 19.8 | 22.7 | 32.0 | -6.9% | 0.66 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 220.7 | 36.2 | 231.3 | 28.1 | 32.4 | 251.8 | +4.8% | 0.33 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q5_p50_us | 173.6 | 39.2 | 155.0 | 12.8 | 29.2 | 275.5 | -10.7% | 0.64 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 34.1 | 2.32 | 42.4 | 7.71 | 5.69 | 23.6 | +24.6% | 1.47 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 1027 | 64.1 | 1079 | 93.9 | 80.4 | 629.2 | +5.0% | 0.64 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 127.8 | 16.8 | 159.8 | 50.5 | 37.6 | 86.8 | +25.1% | 0.85 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 83.4 | 10.8 | 98.7 | 15.4 | 13.3 | 65.5 | +18.3% | 1.15 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2662 | 349.9 | 2763 | 341.5 | 345.7 | 1511 | +3.8% | 0.29 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 235.0 | 34.9 | 222.3 | 23.3 | 29.7 | 33.1 | -5.4% | 0.43 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 12147 | 4806 | 10408 | 6472 | 5700 | 2423 | -14.3% | 0.31 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 1196826 | 12337 | 1193588 | 8859 | 10740 | 38175 | -0.3% | 0.30 | BELOW FLOOR |
| P vs M | write_p99_us | 1321551 | 43105 | 1340656 | 32827 | 38313 | 30832 | +1.4% | 0.50 | BELOW FLOOR |
| P vs M | commits_per_s | 0.83 | 0.01 | 0.84 | 0.01 | 0.01 | 0.03 | +1.1% | 0.97 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 2857 | 718.1 | 2856 | 717.1 | 717.6 | 10925 | -0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 486.6 | 0.83 | 486.7 | 0.78 | 0.80 | 1.03 | +0.0% | 0.11 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 52890 | 12899 | 54395 | 13561 | 13234 | 190643 | +2.8% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 121.1 | 6.17 | 288.2 | 24.8 | 18.1 | 6.57 | +138.0% | 9.23 | **N better** |
| N vs H3 | q1_p99_us | 1292 | 284.9 | 2299 | 1110 | 810.6 | 3086 | +78.0% | 1.24 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 151.4 | 10.1 | 277.0 | 36.1 | 26.5 | 44.7 | +83.0% | 4.73 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 166.4 | 32.7 | 282.9 | 32.6 | 32.7 | 69.6 | +70.0% | 3.57 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q5_p50_us | 621480 | 11217 | 823805 | 16574 | 14151 | 15100 | +32.6% | 14.3 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 78.6 | 24.0 | 259.8 | 58.4 | 44.6 | 32.1 | +230.6% | 4.06 | **N better** |
| N vs H3 | c2_q1_p99_us | 337.7 | 32.9 | 2197 | 433.8 | 307.6 | 1508 | +550.5% | 6.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 97.7 | 28.5 | 253.1 | 75.6 | 57.1 | 86.3 | +159.1% | 2.72 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p50_us | 104.1 | 27.1 | 330.4 | 63.1 | 48.6 | 9.91 | +217.4% | 4.66 | **N better** |
| N vs H3 | c4_q1_p99_us | 1102 | 523.6 | 5997 | 2895 | 2080 | 4212 | +444.3% | 2.35 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 133.7 | 27.9 | 304.1 | 83.3 | 62.2 | 96.6 | +127.4% | 2.74 | no difference |
| N vs H3 | c4_q2_p99_us | 2805 | 1631 | 3346 | 1484 | 1559 | 913.1 | +19.3% | 0.35 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 662.4 | 110.3 | 1694 | 185.5 | 152.6 | 200.3 | +155.7% | 6.76 | **N better** |
| N vs H3 | write_p99_us | 2218 | 387.5 | 5650 | 1883 | 1360 | 1547 | +154.7% | 2.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1216 | 205.9 | 485.9 | 60.4 | 151.7 | 593.3 | -60.0% | 4.81 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | view_state_bytes_per_key_read | 4038 | 982.7 | 193.6 | 1.55 | 694.9 | 10586 | -95.2% | 5.53 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 354.4 | 12.2 | 533.7 | 0.54 | 8.62 | 26.1 | +50.6% | 20.8 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 26218 | 3614 | 64008 | 14383 | 10487 | 171365 | +144.1% | 3.60 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 722.6 | 28.2 | 288.2 | 24.8 | 26.6 | 130.9 | -60.1% | 16.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3669 | 417.7 | 2299 | 1110 | 838.9 | 3369 | -37.3% | 1.63 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 627.0 | 64.3 | 277.0 | 36.1 | 52.2 | 114.5 | -55.8% | 6.71 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 290.9 | 45.1 | 282.9 | 32.6 | 39.4 | 51.2 | -2.7% | 0.20 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q5_p50_us | 820951 | 12224 | 823805 | 16574 | 14562 | 27902 | +0.3% | 0.20 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 671.1 | 87.0 | 259.8 | 58.4 | 74.1 | 39.2 | -61.3% | 5.55 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4510 | 276.2 | 2197 | 433.8 | 363.6 | 1535 | -51.3% | 6.36 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 635.5 | 95.6 | 253.1 | 75.6 | 86.2 | 173.4 | -60.2% | 4.44 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1674 | 157.0 | 330.4 | 63.1 | 119.6 | 115.2 | -80.3% | 11.2 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6616 | 546.9 | 5997 | 2895 | 2083 | 4760 | -9.4% | 0.30 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1556 | 59.6 | 304.1 | 83.3 | 72.4 | 128.3 | -80.5% | 17.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7470 | 1123 | 3346 | 1484 | 1316 | 3815 | -55.2% | 3.13 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1174 | 63.2 | 1694 | 185.5 | 138.6 | 196.5 | +44.3% | 3.75 | **P+ better** |
| P+ vs H3 | write_p99_us | 3533 | 423.9 | 5650 | 1883 | 1365 | 1367 | +59.9% | 1.55 | no difference |
| P+ vs H3 | commits_per_s | 709.1 | 28.1 | 485.9 | 60.4 | 47.1 | 203.1 | -31.5% | 4.74 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | view_state_bytes_per_key_read | 4343 | 1070 | 193.6 | 1.55 | 756.9 | 11473 | -95.5% | 5.48 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 492.2 | 0.49 | 533.7 | 0.54 | 0.52 | 0.56 | +8.4% | 80.3 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 50866 | 12727 | 64008 | 14383 | 13581 | 214510 | +25.8% | 0.97 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 121.4 | 8.36 | 607.1 | 19.5 | 15.0 | 49.6 | +400.1% | 32.4 | **N better** |
| N vs P+ | q1_p99_us | 1292 | 326.9 | 3702 | 330.0 | 328.4 | 963.4 | +186.6% | 7.34 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 153.4 | 9.95 | 519.8 | 16.2 | 13.5 | 158.5 | +238.9% | 27.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 155.6 | 29.7 | 296.3 | 37.9 | 34.1 | 154.7 | +90.4% | 4.13 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | 197692 | 1829 | 76483 | 914.1 | 1446 | 9468 | -61.3% | 83.8 | **P+ better** |
| N vs P+ | q5_p50_us | 628169 | 8095 | 827021 | 10492 | 9371 | 18398 | +31.7% | 21.2 | **N better** |
| N vs P+ | q6_p50_us | 1084972 | 24650 | 851364 | 15977 | 20772 | 28699 | -21.5% | 11.2 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 109.9 | 9.60 | 608.5 | 75.9 | 54.1 | 41.3 | +453.6% | 9.21 | **N better** |
| N vs P+ | c2_q1_p99_us | 451.0 | 156.4 | 4768 | 239.1 | 202.0 | 1686 | +957.2% | 21.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 152.9 | 14.5 | 529.2 | 62.0 | 45.0 | 27.6 | +246.0% | 8.35 | **N better** |
| N vs P+ | c4_q1_p50_us | 132.4 | 33.4 | 1517 | 78.5 | 60.3 | 154.6 | +1046.3% | 23.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1064 | 377.2 | 8329 | 1407 | 1030 | 2594 | +683.1% | 7.05 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 161.7 | 49.1 | 1418 | 96.6 | 76.6 | 232.1 | +777.1% | 16.4 | **N better** |
| N vs P+ | c4_q2_p99_us | 7308 | 4249 | 6060 | 1637 | 3220 | 1900 | -17.1% | 0.39 | BELOW FLOOR |
| N vs P+ | write_p50_us | 703.3 | 113.9 | 1096 | 120.9 | 117.5 | 307.6 | +55.8% | 3.34 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 2495 | 478.9 | 3629 | 735.9 | 620.9 | 1826 | +45.4% | 1.83 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1090 | 260.5 | 750.4 | 97.4 | 196.7 | 449.5 | -31.2% | 1.73 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | view_state_bytes_per_key_read | 4087 | 994.9 | 4397 | 1085 | 1041 | 16196 | +7.6% | 0.30 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 370.2 | 17.3 | 492.8 | 0.06 | 12.2 | 26.3 | +33.1% | 10.0 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 32926 | 3455 | 50182 | 12647 | 9271 | 127607 | +52.4% | 1.86 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 121.4 | 8.36 | 109.9 | 11.5 | 10.0 | 33.1 | -9.5% | 1.14 | BELOW FLOOR |
| N vs P | q1_p99_us | 1292 | 326.9 | 1005 | 435.1 | 384.8 | 630.7 | -22.2% | 0.75 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 153.4 | 9.95 | 222.3 | 15.8 | 13.2 | 67.1 | +45.0% | 5.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 155.6 | 29.7 | 220.2 | 41.9 | 36.3 | 139.8 | +41.5% | 1.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q4_p50_us | 197692 | 1829 | 4874 | 130.6 | 1297 | 9261 | -97.5% | 148.7 | **P better** |
| N vs P | q5_p50_us | 628169 | 8095 | 166.4 | 18.0 | 5724 | 14304 | -100.0% | 109.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 1084972 | 24650 | 28989 | 869.9 | 17441 | 594.4 | -97.3% | 60.5 | **P better** |
| N vs P | c2_q1_p50_us | 109.9 | 9.60 | 36.0 | 3.14 | 7.14 | 63.2 | -67.3% | 10.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 451.0 | 156.4 | 1085 | 107.2 | 134.0 | 2037 | +140.6% | 4.73 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 152.9 | 14.5 | 142.2 | 20.2 | 17.6 | 16.7 | -7.0% | 0.61 | BELOW FLOOR |
| N vs P | c4_q1_p50_us | 132.4 | 33.4 | 109.1 | 28.3 | 31.0 | 108.0 | -17.6% | 0.75 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1064 | 377.2 | 3178 | 1046 | 786.0 | 661.3 | +198.8% | 2.69 | no difference |
| N vs P | c4_q2_p50_us | 161.7 | 49.1 | 268.2 | 57.3 | 53.4 | 55.7 | +65.9% | 2.00 | no difference |
| N vs P | c4_q2_p99_us | 7308 | 4249 | 18634 | 9849 | 7585 | 8887 | +155.0% | 1.49 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 703.3 | 113.9 | 1193203 | 10578 | 7480 | 2522 | +169562.8% | 159.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p99_us | 2495 | 478.9 | 1324872 | 30690 | 21704 | 48398 | +52997.6% | 60.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1090 | 260.5 | 0.83 | 0.01 | 184.2 | 444.8 | -99.9% | 5.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | view_state_bytes_per_key_read | 4087 | 994.9 | 2892 | 727.9 | 871.6 | 13598 | -29.2% | 1.37 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 370.2 | 17.3 | 486.9 | 0.81 | 12.2 | 26.1 | +31.5% | 9.54 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 32926 | 3455 | 54322 | 13333 | 9739 | 134734 | +65.0% | 2.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 607.1 | 19.5 | 109.9 | 11.5 | 16.0 | 59.4 | -81.9% | 31.1 | **P better** |
| P+ vs P | q1_p99_us | 3702 | 330.0 | 1005 | 435.1 | 386.1 | 729.3 | -72.9% | 6.98 | **P better** |
| P+ vs P | q2_p50_us | 519.8 | 16.2 | 222.3 | 15.8 | 16.0 | 146.3 | -57.2% | 18.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 296.3 | 37.9 | 220.2 | 41.9 | 40.0 | 66.3 | -25.7% | 1.90 | no difference |
| P+ vs P | q4_p50_us | 76483 | 914.1 | 4874 | 130.6 | 652.9 | 1972 | -93.6% | 109.7 | **P better** |
| P+ vs P | q5_p50_us | 827021 | 10492 | 166.4 | 18.0 | 7419 | 11571 | -100.0% | 111.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 851364 | 15977 | 28989 | 869.9 | 11314 | 28703 | -96.6% | 72.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 608.5 | 75.9 | 36.0 | 3.14 | 53.7 | 73.9 | -94.1% | 10.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 4768 | 239.1 | 1085 | 107.2 | 185.3 | 2643 | -77.2% | 19.9 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 529.2 | 62.0 | 142.2 | 20.2 | 46.1 | 26.1 | -73.1% | 8.39 | **P better** |
| P+ vs P | c4_q1_p50_us | 1517 | 78.5 | 109.1 | 28.3 | 59.0 | 156.9 | -92.8% | 23.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8329 | 1407 | 3178 | 1046 | 1240 | 2635 | -61.8% | 4.15 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1418 | 96.6 | 268.2 | 57.3 | 79.4 | 232.9 | -81.1% | 14.5 | **P better** |
| P+ vs P | c4_q2_p99_us | 6060 | 1637 | 18634 | 9849 | 7060 | 8795 | +207.5% | 1.78 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 1096 | 120.9 | 1193203 | 10578 | 7480 | 2504 | +108811.2% | 159.4 | **P+ better** |
| P+ vs P | write_p99_us | 3629 | 735.9 | 1324872 | 30690 | 21707 | 48394 | +36406.9% | 60.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 750.4 | 97.4 | 0.83 | 0.01 | 68.9 | 65.0 | -99.9% | 10.9 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 4397 | 1085 | 2892 | 727.9 | 923.9 | 14339 | -34.2% | 1.63 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 492.8 | 0.06 | 486.9 | 0.81 | 0.58 | 3.54 | -1.2% | 10.3 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 50182 | 12647 | 54322 | 13333 | 12995 | 185533 | +8.2% | 0.32 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 109.9 | 11.5 | 108.0 | 8.44 | 10.1 | 57.6 | -1.8% | 0.19 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q1_p99_us | 1005 | 435.1 | 949.2 | 368.2 | 403.0 | 97.2 | -5.6% | 0.14 | BELOW FLOOR |
| P vs M | q2_p50_us | 222.3 | 15.8 | 224.2 | 8.76 | 12.8 | 23.7 | +0.9% | 0.15 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 220.2 | 41.9 | 204.2 | 17.7 | 32.2 | 21.5 | -7.3% | 0.50 | BELOW FLOOR |
| P vs M | q4_p50_us | 4874 | 130.6 | 4836 | 159.1 | 145.5 | 750.3 | -0.8% | 0.27 | BELOW FLOOR |
| P vs M | q5_p50_us | 166.4 | 18.0 | 159.8 | 16.1 | 17.1 | 148.8 | -3.9% | 0.38 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | 28989 | 869.9 | 28830 | 618.5 | 754.7 | 1029 | -0.5% | 0.21 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 36.0 | 3.14 | 33.2 | 1.00 | 2.33 | 62.3 | -7.6% | 1.17 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1085 | 107.2 | 1112 | 102.2 | 104.7 | 2168 | +2.5% | 0.25 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 142.2 | 20.2 | 132.0 | 8.22 | 15.4 | 11.4 | -7.1% | 0.66 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 109.1 | 28.3 | 85.3 | 21.7 | 25.2 | 94.8 | -21.8% | 0.94 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 3178 | 1046 | 2561 | 604.4 | 854.0 | 1787 | -19.4% | 0.72 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 268.2 | 57.3 | 250.9 | 63.1 | 60.3 | 42.3 | -6.4% | 0.29 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 18634 | 9849 | 17034 | 7067 | 8571 | 13643 | -8.6% | 0.19 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 1193203 | 10578 | 1188847 | 9728 | 10162 | 14560 | -0.4% | 0.43 | BELOW FLOOR |
| P vs M | write_p99_us | 1324872 | 30690 | 1296548 | 26562 | 28700 | 49147 | -2.1% | 0.99 | BELOW FLOOR |
| P vs M | commits_per_s | 0.83 | 0.01 | 0.84 | 0.01 | 0.01 | 0.04 | +0.5% | 0.61 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 2892 | 727.9 | 2892 | 727.9 | 727.9 | 11324 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 486.9 | 0.81 | 487.4 | 0.39 | 0.64 | 3.92 | +0.1% | 0.81 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 54322 | 13333 | 54011 | 13483 | 13408 | 193423 | -0.6% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 121.4 | 8.36 | 294.4 | 31.4 | 23.0 | 85.6 | +142.6% | 7.54 | **N better** |
| N vs H3 | q1_p99_us | 1292 | 326.9 | 2664 | 1081 | 798.5 | 5891 | +106.2% | 1.72 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 153.4 | 9.95 | 289.4 | 33.8 | 24.9 | 165.4 | +88.7% | 5.46 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 155.6 | 29.7 | 340.1 | 75.0 | 57.1 | 146.5 | +118.6% | 3.23 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | 197692 | 1829 | 73856 | 3700 | 2918 | 9325 | -62.6% | 42.4 | **H3 better** |
| N vs H3 | q5_p50_us | 628169 | 8095 | 813270 | 9597 | 8878 | 39060 | +29.5% | 20.8 | **N better** |
| N vs H3 | q6_p50_us | 1084972 | 24650 | 867930 | 11766 | 19314 | 4492 | -20.0% | 11.2 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 109.9 | 9.60 | 244.2 | 43.2 | 31.3 | 16.2 | +122.2% | 4.29 | **N better** |
| N vs H3 | c2_q1_p99_us | 451.0 | 156.4 | 2498 | 1294 | 921.7 | 312.3 | +454.0% | 2.22 | no difference |
| N vs H3 | c2_q2_p50_us | 152.9 | 14.5 | 223.3 | 35.8 | 27.3 | 63.0 | +46.0% | 2.58 | no difference |
| N vs H3 | c4_q1_p50_us | 132.4 | 33.4 | 307.4 | 84.9 | 64.5 | 140.3 | +132.2% | 2.71 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 1064 | 377.2 | 4710 | 1532 | 1116 | 2181 | +342.9% | 3.27 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 161.7 | 49.1 | 325.2 | 91.2 | 73.3 | 243.5 | +101.2% | 2.23 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 7308 | 4249 | 5547 | 3605 | 3940 | 6685 | -24.1% | 0.45 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 703.3 | 113.9 | 1768 | 147.3 | 131.6 | 310.3 | +151.4% | 8.09 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p99_us | 2495 | 478.9 | 4641 | 722.4 | 612.8 | 1379 | +86.0% | 3.50 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1090 | 260.5 | 469.2 | 40.4 | 186.4 | 464.5 | -57.0% | 3.33 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | view_state_bytes_per_key_read | 4087 | 994.9 | 193.8 | 2.48 | 703.5 | 10991 | -95.3% | 5.53 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 370.2 | 17.3 | 609.1 | 19.5 | 18.4 | 33.0 | +64.5% | 13.0 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 32926 | 3455 | 85400 | 17872 | 12871 | 159265 | +159.4% | 4.08 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 607.1 | 19.5 | 294.4 | 31.4 | 26.1 | 98.8 | -51.5% | 12.0 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3702 | 330.0 | 2664 | 1081 | 799.2 | 5902 | -28.0% | 1.30 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 519.8 | 16.2 | 289.4 | 33.8 | 26.5 | 210.4 | -44.3% | 8.69 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 296.3 | 37.9 | 340.1 | 75.0 | 59.4 | 79.4 | +14.8% | 0.74 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 76483 | 914.1 | 73856 | 3700 | 2695 | 2251 | -3.4% | 0.97 | no difference |
| P+ vs H3 | q5_p50_us | 827021 | 10492 | 813270 | 9597 | 10055 | 38144 | -1.7% | 1.37 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 851364 | 15977 | 867930 | 11766 | 14030 | 29046 | +1.9% | 1.18 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 608.5 | 75.9 | 244.2 | 43.2 | 61.8 | 41.7 | -59.9% | 5.90 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4768 | 239.1 | 2498 | 1294 | 930.5 | 1713 | -47.6% | 2.44 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p50_us | 529.2 | 62.0 | 223.3 | 35.8 | 50.6 | 66.1 | -57.8% | 6.04 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1517 | 78.5 | 307.4 | 84.9 | 81.8 | 180.6 | -79.7% | 14.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 8329 | 1407 | 4710 | 1532 | 1471 | 3356 | -43.4% | 2.46 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1418 | 96.6 | 325.2 | 91.2 | 94.0 | 332.3 | -77.1% | 11.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 6060 | 1637 | 5547 | 3605 | 2800 | 6563 | -8.5% | 0.18 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1096 | 120.9 | 1768 | 147.3 | 134.7 | 63.4 | +61.4% | 4.99 | **P+ better** |
| P+ vs H3 | write_p99_us | 3629 | 735.9 | 4641 | 722.4 | 729.2 | 1229 | +27.9% | 1.39 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 750.4 | 97.4 | 469.2 | 40.4 | 74.6 | 149.0 | -37.5% | 3.77 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 4397 | 1085 | 193.8 | 2.48 | 767.2 | 11895 | -95.6% | 5.48 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 492.8 | 0.06 | 609.1 | 19.5 | 13.8 | 20.4 | +23.6% | 8.43 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 50182 | 12647 | 85400 | 17872 | 15481 | 204045 | +70.2% | 2.27 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 150.1 | 10.9 | 640.4 | 23.9 | 18.6 | 48.0 | +326.7% | 26.4 | **N better** |
| N vs P+ | q1_p99_us | 1466 | 577.1 | 3568 | 359.3 | 480.7 | 831.3 | +143.4% | 4.37 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 173.4 | 27.8 | 543.5 | 26.1 | 27.0 | 107.8 | +213.5% | 13.7 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 216.1 | 45.6 | 302.7 | 25.0 | 36.8 | 180.0 | +40.0% | 2.35 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q4_p50_us | 200711 | 3722 | 75498 | 2136 | 3035 | 1270 | -62.4% | 41.3 | **P+ better** |
| N vs P+ | q5_p50_us | 627163 | 12484 | 803067 | 7756 | 10392 | 22206 | +28.0% | 16.9 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 97.2 | 25.2 | 601.7 | 93.2 | 68.3 | 24.0 | +519.2% | 7.39 | **N better** |
| N vs P+ | c2_q1_p99_us | 302.7 | 25.8 | 4409 | 569.2 | 402.9 | 426.4 | +1356.6% | 10.2 | **N better** |
| N vs P+ | c2_q2_p50_us | 104.7 | 16.3 | 550.3 | 93.7 | 67.2 | 95.8 | +425.6% | 6.63 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 114.2 | 28.6 | 1645 | 144.5 | 104.2 | 69.5 | +1340.3% | 14.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 792.4 | 137.4 | 8100 | 905.6 | 647.7 | 4199 | +922.1% | 11.3 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 131.8 | 36.0 | 1549 | 94.3 | 71.3 | 236.9 | +1075.7% | 19.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 3054 | 1616 | 7570 | 1353 | 1490 | 3789 | +147.9% | 3.03 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 645.5 | 73.2 | 1071 | 28.4 | 55.5 | 260.3 | +66.0% | 7.67 | **N better** |
| N vs P+ | write_p99_us | 2270 | 497.5 | 3350 | 316.1 | 416.8 | 935.0 | +47.6% | 2.59 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1175 | 245.6 | 772.8 | 39.7 | 175.9 | 323.6 | -34.2% | 2.28 | no difference |
| N vs P+ | view_state_bytes_per_key_read | 4032 | 953.9 | 4316 | 1035 | 995.0 | 15919 | +7.0% | 0.29 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 354.6 | 7.11 | 494.2 | 0.18 | 5.03 | 0.79 | +39.4% | 27.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 26419 | 4618 | 49364 | 12068 | 9137 | 128257 | +86.8% | 2.51 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 150.1 | 10.9 | 111.4 | 4.32 | 8.26 | 36.3 | -25.8% | 4.69 | **P better** |
| N vs P | q1_p99_us | 1466 | 577.1 | 887.3 | 372.6 | 485.7 | 351.1 | -39.5% | 1.19 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 173.4 | 27.8 | 229.8 | 28.1 | 28.0 | 44.6 | +32.6% | 2.02 | no difference |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 216.1 | 45.6 | 228.0 | 12.7 | 33.5 | 16.0 | +5.5% | 0.36 | BELOW FLOOR |
| N vs P | q4_p50_us | 200711 | 3722 | 4908 | 78.0 | 2632 | 1144 | -97.6% | 74.4 | **P better** |
| N vs P | q5_p50_us | 627163 | 12484 | 172.9 | 15.7 | 8827 | 18141 | -100.0% | 71.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 97.2 | 25.2 | 42.6 | 8.14 | 18.7 | 24.2 | -56.1% | 2.91 | no difference |
| N vs P | c2_q1_p99_us | 302.7 | 25.8 | 1095 | 150.9 | 108.3 | 406.9 | +261.7% | 7.32 | **N better** |
| N vs P | c2_q2_p50_us | 104.7 | 16.3 | 143.7 | 33.9 | 26.6 | 124.5 | +37.2% | 1.46 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p50_us | 114.2 | 28.6 | 99.2 | 18.7 | 24.2 | 91.7 | -13.2% | 0.62 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 792.4 | 137.4 | 2338 | 410.6 | 306.2 | 1029 | +195.0% | 5.05 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 131.8 | 36.0 | 228.4 | 54.9 | 46.4 | 125.1 | +73.3% | 2.08 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 3054 | 1616 | 9770 | 3136 | 2494 | 8801 | +219.9% | 2.69 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 645.5 | 73.2 | 1173770 | 9221 | 6520 | 7243 | +181735.4% | 179.9 | **N better** |
| N vs P | write_p99_us | 2270 | 497.5 | 1278572 | 30904 | 21855 | 43220 | +56230.0% | 58.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1175 | 245.6 | 0.85 | 0.01 | 173.6 | 313.9 | -99.9% | 6.76 | **N better** |
| N vs P | view_state_bytes_per_key_read | 4032 | 953.9 | 2851 | 698.3 | 835.9 | 13397 | -29.3% | 1.41 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 354.6 | 7.11 | 488.6 | 0.89 | 5.06 | 0.53 | +37.8% | 26.5 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 26419 | 4618 | 52049 | 12584 | 9478 | 130293 | +97.0% | 2.70 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 640.4 | 23.9 | 111.4 | 4.32 | 17.2 | 55.8 | -82.6% | 30.8 | **P better** |
| P+ vs P | q1_p99_us | 3568 | 359.3 | 887.3 | 372.6 | 366.0 | 772.2 | -75.1% | 7.33 | **P better** |
| P+ vs P | q2_p50_us | 543.5 | 26.1 | 229.8 | 28.1 | 27.2 | 104.6 | -57.7% | 11.5 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 302.7 | 25.0 | 228.0 | 12.7 | 19.8 | 179.7 | -24.7% | 3.77 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q4_p50_us | 75498 | 2136 | 4908 | 78.0 | 1512 | 891.1 | -93.5% | 46.7 | **P better** |
| P+ vs P | q5_p50_us | 803067 | 7756 | 172.9 | 15.7 | 5484 | 12808 | -100.0% | 146.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 601.7 | 93.2 | 42.6 | 8.14 | 66.2 | 31.7 | -92.9% | 8.45 | **P better** |
| P+ vs P | c2_q1_p99_us | 4409 | 569.2 | 1095 | 150.9 | 416.4 | 589.2 | -75.2% | 7.96 | **P better** |
| P+ vs P | c2_q2_p50_us | 550.3 | 93.7 | 143.7 | 33.9 | 70.4 | 80.5 | -73.9% | 5.77 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1645 | 144.5 | 99.2 | 18.7 | 103.0 | 78.1 | -94.0% | 15.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8100 | 905.6 | 2338 | 410.6 | 703.1 | 4196 | -71.1% | 8.19 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1549 | 94.3 | 228.4 | 54.9 | 77.1 | 201.2 | -85.3% | 17.1 | **P better** |
| P+ vs P | c4_q2_p99_us | 7570 | 1353 | 9770 | 3136 | 2415 | 8306 | +29.1% | 0.91 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 1071 | 28.4 | 1173770 | 9221 | 6520 | 7243 | +109448.1% | 179.9 | **P+ better** |
| P+ vs P | write_p99_us | 3350 | 316.1 | 1278572 | 30904 | 21854 | 43211 | +38064.9% | 58.4 | **P+ better** |
| P+ vs P | commits_per_s | 772.8 | 39.7 | 0.85 | 0.01 | 28.1 | 78.5 | -99.9% | 27.5 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 4316 | 1035 | 2851 | 698.3 | 882.6 | 14093 | -33.9% | 1.66 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 494.2 | 0.18 | 488.6 | 0.89 | 0.64 | 0.90 | -1.1% | 8.86 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 49364 | 12068 | 52049 | 12584 | 12329 | 182784 | +5.4% | 0.22 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 111.4 | 4.32 | 108.1 | 7.32 | 6.01 | 72.9 | -3.0% | 0.55 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q1_p99_us | 887.3 | 372.6 | 1209 | 613.8 | 507.7 | 276.3 | +36.2% | 0.63 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 229.8 | 28.1 | 231.2 | 24.1 | 26.2 | 70.6 | +0.6% | 0.05 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 228.0 | 12.7 | 211.3 | 26.0 | 20.5 | 68.6 | -7.3% | 0.82 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 4908 | 78.0 | 4893 | 152.8 | 121.3 | 496.2 | -0.3% | 0.12 | BELOW FLOOR |
| P vs M | q5_p50_us | 172.9 | 15.7 | 192.4 | 43.1 | 32.4 | 122.3 | +11.2% | 0.60 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 42.6 | 8.14 | 36.1 | 3.55 | 6.28 | 27.9 | -15.4% | 1.04 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1095 | 150.9 | 1129 | 119.7 | 136.2 | 456.6 | +3.1% | 0.25 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 143.7 | 33.9 | 135.2 | 11.6 | 25.3 | 96.1 | -5.9% | 0.33 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 99.2 | 18.7 | 108.1 | 18.2 | 18.5 | 83.5 | +9.0% | 0.48 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2338 | 410.6 | 2719 | 437.0 | 424.0 | 761.9 | +16.3% | 0.90 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 228.4 | 54.9 | 282.3 | 29.8 | 44.1 | 61.6 | +23.6% | 1.22 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 9770 | 3136 | 10575 | 3339 | 3239 | 9018 | +8.2% | 0.25 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 1173770 | 9221 | 1181267 | 11929 | 10661 | 13868 | +0.6% | 0.70 | BELOW FLOOR |
| P vs M | write_p99_us | 1278572 | 30904 | 1304160 | 35024 | 33029 | 43239 | +2.0% | 0.77 | BELOW FLOOR |
| P vs M | commits_per_s | 0.85 | 0.01 | 0.84 | 0.01 | 0.01 | 0.01 | -0.4% | 0.34 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 2851 | 698.3 | 2851 | 698.3 | 698.3 | 11166 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 488.6 | 0.89 | 488.8 | 0.73 | 0.81 | 0.86 | +0.1% | 0.31 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 52049 | 12584 | 53329 | 12928 | 12757 | 188796 | +2.5% | 0.10 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 150.1 | 10.9 | 293.6 | 28.5 | 21.5 | 56.0 | +95.6% | 6.66 | **N better** |
| N vs H3 | q1_p99_us | 1466 | 577.1 | 2811 | 849.3 | 726.1 | 978.7 | +91.7% | 1.85 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 173.4 | 27.8 | 349.7 | 88.6 | 65.7 | 57.7 | +101.7% | 2.68 | no difference |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 216.1 | 45.6 | 340.5 | 37.8 | 41.9 | 69.6 | +57.5% | 2.97 | no difference |
| N vs H3 | q4_p50_us | 200711 | 3722 | 72903 | 1574 | 2858 | 2610 | -63.7% | 44.7 | **H3 better** |
| N vs H3 | q5_p50_us | 627163 | 12484 | 801597 | 10759 | 11653 | 18149 | +27.8% | 15.0 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 97.2 | 25.2 | 237.0 | 32.8 | 29.3 | 140.4 | +143.8% | 4.77 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 302.7 | 25.8 | 2264 | 1054 | 745.2 | 6159 | +648.0% | 2.63 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 104.7 | 16.3 | 210.2 | 35.4 | 27.5 | 96.2 | +100.7% | 3.83 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 114.2 | 28.6 | 333.4 | 66.9 | 51.5 | 68.2 | +191.8% | 4.26 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 792.4 | 137.4 | 4348 | 1430 | 1016 | 1545 | +448.7% | 3.50 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 131.8 | 36.0 | 325.2 | 75.1 | 58.8 | 131.4 | +146.8% | 3.29 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 3054 | 1616 | 4091 | 2211 | 1936 | 6402 | +34.0% | 0.54 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 645.5 | 73.2 | 1671 | 119.6 | 99.2 | 449.4 | +158.9% | 10.3 | **N better** |
| N vs H3 | write_p99_us | 2270 | 497.5 | 10102 | 5236 | 3719 | 3468 | +345.1% | 2.11 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1175 | 245.6 | 432.6 | 72.0 | 180.9 | 313.9 | -63.2% | 4.10 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 4032 | 953.9 | 193.8 | 1.14 | 674.5 | 10823 | -95.2% | 5.69 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 354.6 | 7.11 | 573.7 | 20.4 | 15.3 | 0.44 | +61.8% | 14.4 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 26419 | 4618 | 75634 | 15190 | 11226 | 180092 | +186.3% | 4.38 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 640.4 | 23.9 | 293.6 | 28.5 | 26.3 | 70.2 | -54.1% | 13.2 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3568 | 359.3 | 2811 | 849.3 | 652.1 | 1196 | -21.2% | 1.16 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 543.5 | 26.1 | 349.7 | 88.6 | 65.3 | 110.9 | -35.7% | 2.97 | no difference |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 302.7 | 25.0 | 340.5 | 37.8 | 32.0 | 192.1 | +12.5% | 1.18 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | 75498 | 2136 | 72903 | 1574 | 1876 | 2509 | -3.4% | 1.38 | no difference |
| P+ vs H3 | q5_p50_us | 803067 | 7756 | 801597 | 10759 | 9379 | 12820 | -0.2% | 0.16 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 601.7 | 93.2 | 237.0 | 32.8 | 69.9 | 141.9 | -60.6% | 5.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 4409 | 569.2 | 2264 | 1054 | 846.7 | 6174 | -48.7% | 2.53 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 550.3 | 93.7 | 210.2 | 35.4 | 70.8 | 14.7 | -61.8% | 4.81 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1645 | 144.5 | 333.4 | 66.9 | 112.6 | 48.5 | -79.7% | 11.7 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 8100 | 905.6 | 4348 | 1430 | 1197 | 4351 | -46.3% | 3.13 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1549 | 94.3 | 325.2 | 75.1 | 85.2 | 205.2 | -79.0% | 14.4 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7570 | 1353 | 4091 | 2211 | 1833 | 5703 | -46.0% | 1.90 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1071 | 28.4 | 1671 | 119.6 | 86.9 | 446.6 | +56.0% | 6.90 | **P+ better** |
| P+ vs H3 | write_p99_us | 3350 | 316.1 | 10102 | 5236 | 3709 | 3345 | +201.6% | 1.82 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 772.8 | 39.7 | 432.6 | 72.0 | 58.1 | 78.5 | -44.0% | 5.85 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 4316 | 1035 | 193.8 | 1.14 | 731.5 | 11673 | -95.5% | 5.63 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 494.2 | 0.18 | 573.7 | 20.4 | 14.4 | 0.84 | +16.1% | 5.52 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 49364 | 12068 | 75634 | 15190 | 13718 | 221059 | +53.2% | 1.92 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

## `single`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 131.0 | 14.6 | 666.9 | 30.6 | 24.0 | 144.7 | +408.9% | 22.3 | **N better** |
| N vs P+ | q1_p99_us | 1508 | 304.1 | 3907 | 537.2 | 436.5 | 839.6 | +159.1% | 5.50 | **N better** |
| N vs P+ | q2_p50_us | 162.2 | 18.7 | 569.6 | 57.4 | 42.7 | 211.1 | +251.2% | 9.55 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | 182.4 | 33.5 | 303.9 | 47.7 | 41.2 | 112.1 | +66.6% | 2.95 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q4_p50_us | 200681 | 2891 | 79063 | 4476 | 3768 | 35556 | -60.6% | 32.3 | **P+ better** |
| N vs P+ | q5_p50_us | 632871 | 4460 | 831516 | 8859 | 7014 | 40265 | +31.4% | 28.3 | **N better** |
| N vs P+ | q6_p50_us | 1118842 | 33800 | 856916 | 31209 | 32530 | 60517 | -23.4% | 8.05 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 72.9 | 25.7 | 632.1 | 76.1 | 56.8 | 271.8 | +767.6% | 9.85 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q1_p99_us | 298.0 | 40.3 | 5254 | 1478 | 1046 | 1179 | +1662.8% | 4.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 95.6 | 43.7 | 588.6 | 53.8 | 49.0 | 284.0 | +515.7% | 10.1 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q1_p50_us | 89.2 | 5.03 | 1596 | 85.3 | 60.4 | 771.8 | +1690.3% | 24.9 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q1_p99_us | 906.3 | 551.2 | 7205 | 758.0 | 662.7 | 5697 | +695.0% | 9.50 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 121.3 | 14.9 | 1505 | 125.7 | 89.5 | 334.9 | +1141.1% | 15.5 | **N better** |
| N vs P+ | c4_q2_p99_us | 5332 | 3057 | 8785 | 2190 | 2659 | 6048 | +64.8% | 1.30 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 703.6 | 136.7 | 1132 | 82.2 | 112.8 | 153.6 | +60.9% | 3.80 | **N better** |
| N vs P+ | write_p99_us | 2170 | 385.1 | 3557 | 554.9 | 477.6 | 1236 | +63.9% | 2.90 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1221 | 126.3 | 725.0 | 61.5 | 99.3 | 242.8 | -40.6% | 5.00 | **N better** |
| N vs P+ | view_state_bytes_per_key_read | 4084 | 977.4 | 4388 | 1062 | 1021 | 15783 | +7.4% | 0.30 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 348.6 | 16.9 | 493.8 | 0.11 | 12.0 | 4.82 | +41.7% | 12.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 19259 | 2873 | 51021 | 12560 | 9111 | 120978 | +164.9% | 3.49 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 131.0 | 14.6 | 114.9 | 5.05 | 11.0 | 12.8 | -12.3% | 1.47 | no difference |
| N vs P | q1_p99_us | 1508 | 304.1 | 1356 | 569.7 | 456.6 | 307.4 | -10.1% | 0.33 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 162.2 | 18.7 | 241.1 | 16.8 | 17.8 | 59.1 | +48.7% | 4.44 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | 182.4 | 33.5 | 261.1 | 44.6 | 39.4 | 114.4 | +43.1% | 2.00 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q4_p50_us | 200681 | 2891 | 4978 | 171.2 | 2048 | 34958 | -97.5% | 95.6 | **P better** |
| N vs P | q5_p50_us | 632871 | 4460 | 182.9 | 18.7 | 3154 | 39287 | -100.0% | 200.6 | **P better** |
| N vs P | q6_p50_us | 1118842 | 33800 | 29584 | 884.8 | 23908 | 59363 | -97.4% | 45.6 | **P better** |
| N vs P | c2_q1_p50_us | 72.9 | 25.7 | 41.8 | 8.74 | 19.2 | 48.6 | -42.6% | 1.62 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 298.0 | 40.3 | 1083 | 345.8 | 246.2 | 800.9 | +263.2% | 3.19 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 95.6 | 43.7 | 159.4 | 31.6 | 38.1 | 179.3 | +66.8% | 1.67 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p50_us | 89.2 | 5.03 | 106.9 | 30.7 | 22.0 | 66.6 | +19.9% | 0.81 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 906.3 | 551.2 | 3604 | 609.6 | 581.2 | 1021 | +297.6% | 4.64 | **N better** |
| N vs P | c4_q2_p50_us | 121.3 | 14.9 | 266.6 | 33.2 | 25.7 | 113.9 | +119.8% | 5.64 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 5332 | 3057 | 15464 | 5177 | 4251 | 11844 | +190.0% | 2.38 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 703.6 | 136.7 | 1214563 | 7077 | 5005 | 810.5 | +172511.0% | 242.5 | **N better** |
| N vs P | write_p99_us | 2170 | 385.1 | 1346634 | 22576 | 15966 | 205183 | +61952.0% | 84.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1221 | 126.3 | 0.82 | 0.00 | 89.3 | 239.5 | -99.9% | 13.7 | **N better** |
| N vs P | view_state_bytes_per_key_read | 4084 | 977.4 | 2889 | 712.2 | 855.1 | 13242 | -29.3% | 1.40 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | pss_mib | 348.6 | 16.9 | 487.6 | 1.68 | 12.0 | 3.79 | +39.9% | 11.5 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 19259 | 2873 | 54615 | 13423 | 9706 | 134600 | +183.6% | 3.64 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 666.9 | 30.6 | 114.9 | 5.05 | 22.0 | 144.2 | -82.8% | 25.1 | **P better** |
| P+ vs P | q1_p99_us | 3907 | 537.2 | 1356 | 569.7 | 553.7 | 894.1 | -65.3% | 4.61 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 569.6 | 57.4 | 241.1 | 16.8 | 42.3 | 203.5 | -57.7% | 7.77 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 303.9 | 47.7 | 261.1 | 44.6 | 46.1 | 41.2 | -14.1% | 0.93 | no difference |
| P+ vs P | q4_p50_us | 79063 | 4476 | 4978 | 171.2 | 3167 | 6493 | -93.7% | 23.4 | **P better** |
| P+ vs P | q5_p50_us | 831516 | 8859 | 182.9 | 18.7 | 6265 | 8822 | -100.0% | 132.7 | **P better** |
| P+ vs P | q6_p50_us | 856916 | 31209 | 29584 | 884.8 | 22077 | 12396 | -96.5% | 37.5 | **P better** |
| P+ vs P | c2_q1_p50_us | 632.1 | 76.1 | 41.8 | 8.74 | 54.1 | 268.8 | -93.4% | 10.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q1_p99_us | 5254 | 1478 | 1083 | 345.8 | 1073 | 893.3 | -79.4% | 3.89 | **P better** |
| P+ vs P | c2_q2_p50_us | 588.6 | 53.8 | 159.4 | 31.6 | 44.1 | 260.8 | -72.9% | 9.72 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p50_us | 1596 | 85.3 | 106.9 | 30.7 | 64.1 | 769.6 | -93.3% | 23.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p99_us | 7205 | 758.0 | 3604 | 609.6 | 687.8 | 5766 | -50.0% | 5.24 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1505 | 125.7 | 266.6 | 33.2 | 91.9 | 345.1 | -82.3% | 13.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 8785 | 2190 | 15464 | 5177 | 3975 | 13298 | +76.0% | 1.68 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 1132 | 82.2 | 1214563 | 7077 | 5005 | 795.8 | +107172.8% | 242.5 | **P+ better** |
| P+ vs P | write_p99_us | 3557 | 554.9 | 1346634 | 22576 | 15968 | 205179 | +37759.3% | 84.1 | **P+ better** |
| P+ vs P | commits_per_s | 725.0 | 61.5 | 0.82 | 0.00 | 43.5 | 40.2 | -99.9% | 16.6 | **P+ better** |
| P+ vs P | view_state_bytes_per_key_read | 4388 | 1062 | 2889 | 712.2 | 904.2 | 13993 | -34.2% | 1.66 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | pss_mib | 493.8 | 0.11 | 487.6 | 1.68 | 1.19 | 3.37 | -1.3% | 5.22 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 51021 | 12560 | 54615 | 13423 | 12999 | 176975 | +7.0% | 0.28 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 114.9 | 5.05 | 112.0 | 9.56 | 7.65 | 22.3 | -2.6% | 0.39 | BELOW FLOOR |
| P vs M | q1_p99_us | 1356 | 569.7 | 1261 | 543.4 | 556.7 | 343.7 | -7.0% | 0.17 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 241.1 | 16.8 | 239.4 | 20.1 | 18.5 | 63.0 | -0.7% | 0.09 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 261.1 | 44.6 | 278.4 | 41.6 | 43.1 | 110.8 | +6.7% | 0.40 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 4978 | 171.2 | 4895 | 110.1 | 143.9 | 608.0 | -1.7% | 0.58 | BELOW FLOOR |
| P vs M | q5_p50_us | 182.9 | 18.7 | 173.7 | 22.7 | 20.8 | 58.5 | -5.0% | 0.44 | BELOW FLOOR |
| P vs M | q6_p50_us | 29584 | 884.8 | 29193 | 517.4 | 724.8 | 3024 | -1.3% | 0.54 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 41.8 | 8.74 | 34.7 | 1.88 | 6.32 | 20.0 | -17.1% | 1.13 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1083 | 345.8 | 1046 | 72.8 | 249.9 | 155.6 | -3.4% | 0.15 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 159.4 | 31.6 | 143.9 | 27.3 | 29.5 | 103.2 | -9.7% | 0.52 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 106.9 | 30.7 | 78.8 | 32.4 | 31.6 | 97.6 | -26.3% | 0.89 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 3604 | 609.6 | 3031 | 776.7 | 698.2 | 1587 | -15.9% | 0.82 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 266.6 | 33.2 | 236.7 | 32.7 | 33.0 | 134.2 | -11.2% | 0.91 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 15464 | 5177 | 13165 | 5865 | 5532 | 11862 | -14.9% | 0.42 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 1214563 | 7077 | 1221137 | 14637 | 11496 | 21482 | +0.5% | 0.57 | BELOW FLOOR |
| P vs M | write_p99_us | 1346634 | 22576 | 1419831 | 88821 | 64803 | 207151 | +5.4% | 1.13 | BELOW FLOOR |
| P vs M | commits_per_s | 0.82 | 0.00 | 0.81 | 0.01 | 0.01 | 0.01 | -1.2% | 1.22 | BELOW FLOOR |
| P vs M | view_state_bytes_per_key_read | 2889 | 712.2 | 2889 | 712.2 | 712.2 | 11047 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | pss_mib | 487.6 | 1.68 | 487.5 | 0.48 | 1.23 | 1.25 | -0.0% | 0.12 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 54615 | 13423 | 54456 | 13449 | 13436 | 186409 | -0.3% | 0.01 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 131.0 | 14.6 | 327.6 | 43.9 | 32.7 | 80.5 | +150.0% | 6.01 | **N better** |
| N vs H3 | q1_p99_us | 1508 | 304.1 | 2199 | 774.0 | 588.0 | 922.6 | +45.8% | 1.17 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 162.2 | 18.7 | 338.1 | 63.6 | 46.9 | 62.1 | +108.4% | 3.75 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | 182.4 | 33.5 | 313.7 | 52.9 | 44.3 | 154.3 | +72.0% | 2.97 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q4_p50_us | 200681 | 2891 | 76557 | 3232 | 3066 | 35186 | -61.9% | 40.5 | **H3 better** |
| N vs H3 | q5_p50_us | 632871 | 4460 | 834831 | 10973 | 8376 | 44457 | +31.9% | 24.1 | **N better** |
| N vs H3 | q6_p50_us | 1118842 | 33800 | 865149 | 20505 | 27954 | 62401 | -22.7% | 9.08 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 72.9 | 25.7 | 235.6 | 83.6 | 61.9 | 131.9 | +223.3% | 2.63 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 298.0 | 40.3 | 2221 | 1188 | 840.4 | 1829 | +645.2% | 2.29 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 95.6 | 43.7 | 242.5 | 76.9 | 62.5 | 220.4 | +153.7% | 2.35 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p50_us | 89.2 | 5.03 | 281.9 | 73.3 | 52.0 | 67.4 | +216.2% | 3.71 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 906.3 | 551.2 | 5794 | 895.0 | 743.2 | 1963 | +539.3% | 6.58 | **N better** |
| N vs H3 | c4_q2_p50_us | 121.3 | 14.9 | 240.8 | 72.7 | 52.4 | 119.2 | +98.5% | 2.28 | no difference |
| N vs H3 | c4_q2_p99_us | 5332 | 3057 | 5815 | 992.1 | 2272 | 3709 | +9.1% | 0.21 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 703.6 | 136.7 | 1695 | 59.3 | 105.3 | 277.3 | +140.9% | 9.41 | **N better** |
| N vs H3 | write_p99_us | 2170 | 385.1 | 5516 | 1768 | 1280 | 1766 | +154.2% | 2.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1221 | 126.3 | 460.0 | 69.9 | 102.1 | 252.9 | -62.3% | 7.46 | **N better** |
| N vs H3 | view_state_bytes_per_key_read | 4084 | 977.4 | 194.3 | 1.73 | 691.1 | 10692 | -95.2% | 5.63 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 348.6 | 16.9 | 572.4 | 0.49 | 12.0 | 3.68 | +64.2% | 18.7 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 19259 | 2873 | 75262 | 15806 | 11359 | 170034 | +290.8% | 4.93 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 666.9 | 30.6 | 327.6 | 43.9 | 37.9 | 164.6 | -50.9% | 8.96 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3907 | 537.2 | 2199 | 774.0 | 666.2 | 1247 | -43.7% | 2.56 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 569.6 | 57.4 | 338.1 | 63.6 | 60.6 | 204.4 | -40.6% | 3.82 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 303.9 | 47.7 | 313.7 | 52.9 | 50.4 | 111.5 | +3.2% | 0.19 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 79063 | 4476 | 76557 | 3232 | 3904 | 7626 | -3.2% | 0.64 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 831516 | 8859 | 834831 | 10973 | 9973 | 22600 | +0.4% | 0.33 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 856916 | 31209 | 865149 | 20505 | 26405 | 22881 | +1.0% | 0.31 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 632.1 | 76.1 | 235.6 | 83.6 | 80.0 | 295.5 | -62.7% | 4.96 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q1_p99_us | 5254 | 1478 | 2221 | 1188 | 1341 | 1871 | -57.7% | 2.26 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 588.6 | 53.8 | 242.5 | 76.9 | 66.4 | 290.6 | -58.8% | 5.22 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q1_p50_us | 1596 | 85.3 | 281.9 | 73.3 | 79.5 | 769.7 | -82.3% | 16.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p99_us | 7205 | 758.0 | 5794 | 895.0 | 829.3 | 6005 | -19.6% | 1.70 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1505 | 125.7 | 240.8 | 72.7 | 102.6 | 346.9 | -84.0% | 12.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8785 | 2190 | 5815 | 992.1 | 1700 | 7091 | -33.8% | 1.75 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1132 | 82.2 | 1695 | 59.3 | 71.6 | 230.9 | +49.7% | 7.86 | **P+ better** |
| P+ vs H3 | write_p99_us | 3557 | 554.9 | 5516 | 1768 | 1311 | 1310 | +55.1% | 1.49 | no difference |
| P+ vs H3 | commits_per_s | 725.0 | 61.5 | 460.0 | 69.9 | 65.8 | 90.7 | -36.5% | 4.02 | **P+ better** |
| P+ vs H3 | view_state_bytes_per_key_read | 4388 | 1062 | 194.3 | 1.73 | 751.0 | 11609 | -95.6% | 5.58 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | pss_mib | 493.8 | 0.11 | 572.4 | 0.49 | 0.36 | 3.24 | +15.9% | 219.4 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 51021 | 12560 | 75262 | 15806 | 14276 | 205218 | +47.5% | 1.70 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |

