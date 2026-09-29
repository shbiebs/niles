# E27 — every cell

*Generated with `results/E27-comparator.md`, from the same point files. One row per (pair, metric) per point: medians and MADs over the ten measured runs, pooled MAD, floor (3 × pooled MAD of the warm-up runs), relative change (B − A)/A, MADs apart, verdict and, when refused, why.*

## `multi`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 191.7 | 16.9 | 780.9 | 66.6 | 48.6 | 7.73 | +307.3% | 12.1 | **N better** |
| N vs P+ | q1_p99_us | 765.6 | 158.6 | 2666 | 369.1 | 284.1 | 821.9 | +248.2% | 6.69 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 9701 | 330.0 | 577.6 | 43.6 | 235.4 | 858.8 | -94.0% | 38.8 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2249 | 269.1 | 3833 | 197.1 | 235.9 | 703.1 | +70.4% | 6.72 | **N better** |
| N vs P+ | q6_p50_us | 4011 | 222.6 | 4809 | 375.2 | 308.5 | 1185 | +19.9% | 2.59 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 150.9 | 23.2 | 999.6 | 77.8 | 57.4 | 245.8 | +562.3% | 14.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 2552 | 1182 | 6878 | 861.7 | 1034 | 860.0 | +169.4% | 4.18 | **N better** |
| N vs P+ | c2_q2_p50_us | 10196 | 134.8 | 822.9 | 91.7 | 115.3 | 642.0 | -91.9% | 81.3 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 189.2 | 30.6 | 2029 | 142.4 | 103.0 | 353.0 | +972.6% | 17.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 6292 | 1265 | 12638 | 3757 | 2803 | 5041 | +100.9% | 2.26 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 18357 | 1755 | 1868 | 133.3 | 1245 | 384.4 | -89.8% | 13.2 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 43399 | 5353 | 10065 | 2764 | 4260 | 4839 | -76.8% | 7.83 | **P+ better** |
| N vs P+ | write_p50_us | 614.7 | 43.2 | 1127 | 85.5 | 67.8 | 58.5 | +83.4% | 7.56 | **N better** |
| N vs P+ | write_p99_us | 1244 | 446.8 | 4259 | 707.9 | 591.9 | 380.0 | +242.4% | 5.09 | **N better** |
| N vs P+ | commits_per_s | 1468 | 99.5 | 685.3 | 67.8 | 85.1 | 219.1 | -53.3% | 9.20 | **N better** |
| N vs P+ | pss_mib | 22.9 | 2.90 | 43.3 | 0.01 | 2.05 | 0.10 | +89.7% | 10.00 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 18195 | 2069 | -14739 | 1014 | 1629 | 14378 | -181.0% | 20.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 191.7 | 16.9 | 148.1 | 14.0 | 15.5 | 24.4 | -22.7% | 2.81 | no difference |
| N vs P | q1_p99_us | 765.6 | 158.6 | 698.5 | 184.0 | 171.8 | 119.1 | -8.8% | 0.39 | BELOW FLOOR |
| N vs P | q2_p50_us | 9701 | 330.0 | 231.3 | 8.30 | 233.5 | 856.9 | -97.6% | 40.6 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2249 | 269.1 | 169.6 | 19.5 | 190.8 | 263.6 | -92.5% | 10.9 | **P better** |
| N vs P | q6_p50_us | 4011 | 222.6 | 796.6 | 70.4 | 165.1 | 130.3 | -80.1% | 19.5 | **P better** |
| N vs P | c2_q1_p50_us | 150.9 | 23.2 | 152.7 | 53.6 | 41.3 | 145.2 | +1.2% | 0.04 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 2552 | 1182 | 1740 | 297.9 | 861.9 | 492.0 | -31.8% | 0.94 | no difference |
| N vs P | c2_q2_p50_us | 10196 | 134.8 | 274.2 | 44.7 | 100.4 | 646.7 | -97.3% | 98.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 189.2 | 30.6 | 194.8 | 30.6 | 30.6 | 62.7 | +3.0% | 0.18 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 6292 | 1265 | 2361 | 452.6 | 949.8 | 2622 | -62.5% | 4.14 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 18357 | 1755 | 338.9 | 48.0 | 1241 | 385.6 | -98.2% | 14.5 | **P better** |
| N vs P | c4_q2_p99_us | 43399 | 5353 | 3580 | 894.5 | 3837 | 4872 | -91.7% | 10.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 614.7 | 43.2 | 19818 | 613.0 | 434.6 | 2533 | +3123.7% | 44.2 | **N better** |
| N vs P | write_p99_us | 1244 | 446.8 | 33402 | 4778 | 3394 | 322.6 | +2585.4% | 9.48 | **N better** |
| N vs P | commits_per_s | 1468 | 99.5 | 46.0 | 2.23 | 70.4 | 201.1 | -96.9% | 20.2 | **N better** |
| N vs P | pss_mib | 22.9 | 2.90 | 43.3 | 0.04 | 2.05 | 0.13 | +89.5% | 9.98 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 18195 | 2069 | -7647 | 512.4 | 1507 | 7825 | -142.0% | 17.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 780.9 | 66.6 | 148.1 | 14.0 | 48.2 | 25.1 | -81.0% | 13.1 | **P better** |
| P+ vs P | q1_p99_us | 2666 | 369.1 | 698.5 | 184.0 | 291.7 | 815.5 | -73.8% | 6.75 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 577.6 | 43.6 | 231.3 | 8.30 | 31.4 | 57.5 | -60.0% | 11.0 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 373.8 | 61.5 | 280.0 | 32.4 | 49.1 | 220.7 | -25.1% | 1.91 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 1971 | 187.1 | 326.5 | 58.1 | 138.5 | 172.6 | -83.4% | 11.9 | **P better** |
| P+ vs P | q5_p50_us | 3833 | 197.1 | 169.6 | 19.5 | 140.1 | 651.9 | -95.6% | 26.2 | **P better** |
| P+ vs P | q6_p50_us | 4809 | 375.2 | 796.6 | 70.4 | 269.9 | 1189 | -83.4% | 14.9 | **P better** |
| P+ vs P | c2_q1_p50_us | 999.6 | 77.8 | 152.7 | 53.6 | 66.8 | 260.4 | -84.7% | 12.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 6878 | 861.7 | 1740 | 297.9 | 644.7 | 923.9 | -74.7% | 7.97 | **P better** |
| P+ vs P | c2_q2_p50_us | 822.9 | 91.7 | 274.2 | 44.7 | 72.1 | 245.1 | -66.7% | 7.61 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 2029 | 142.4 | 194.8 | 30.6 | 103.0 | 356.5 | -90.4% | 17.8 | **P better** |
| P+ vs P | c4_q1_p99_us | 12638 | 3757 | 2361 | 452.6 | 2676 | 5368 | -81.3% | 3.84 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1868 | 133.3 | 338.9 | 48.0 | 100.2 | 96.6 | -81.9% | 15.3 | **P better** |
| P+ vs P | c4_q2_p99_us | 10065 | 2764 | 3580 | 894.5 | 2054 | 1324 | -64.4% | 3.16 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 1127 | 85.5 | 19818 | 613.0 | 437.7 | 2533 | +1658.2% | 42.7 | **P+ better** |
| P+ vs P | write_p99_us | 4259 | 707.9 | 33402 | 4778 | 3416 | 498.4 | +684.3% | 8.53 | **P+ better** |
| P+ vs P | commits_per_s | 685.3 | 67.8 | 46.0 | 2.23 | 47.9 | 86.9 | -93.3% | 13.3 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.01 | 43.3 | 0.04 | 0.03 | 0.10 | -0.1% | 0.76 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -14739 | 1014 | -7647 | 512.4 | 803.4 | 15767 | -48.1% | 8.83 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 148.1 | 14.0 | 149.2 | 7.35 | 11.2 | 25.2 | +0.7% | 0.10 | BELOW FLOOR |
| P vs M | q1_p99_us | 698.5 | 184.0 | 1003 | 485.0 | 366.8 | 163.5 | +43.6% | 0.83 | no difference |
| P vs M | q2_p50_us | 231.3 | 8.30 | 245.5 | 14.6 | 11.9 | 11.6 | +6.1% | 1.20 | no difference |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 280.0 | 32.4 | 274.4 | 46.9 | 40.3 | 342.8 | -2.0% | 0.14 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 326.5 | 58.1 | 297.2 | 38.0 | 49.1 | 48.1 | -9.0% | 0.60 | BELOW FLOOR |
| P vs M | q5_p50_us | 169.6 | 19.5 | 192.1 | 32.1 | 26.6 | 41.8 | +13.2% | 0.84 | BELOW FLOOR |
| P vs M | q6_p50_us | 796.6 | 70.4 | 829.6 | 101.2 | 87.2 | 584.5 | +4.1% | 0.38 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p50_us | 152.7 | 53.6 | 119.9 | 31.6 | 44.0 | 131.9 | -21.5% | 0.74 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 1740 | 297.9 | 1689 | 354.9 | 327.6 | 1078 | -2.9% | 0.16 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 274.2 | 44.7 | 216.8 | 52.9 | 48.9 | 196.9 | -20.9% | 1.17 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p50_us | 194.8 | 30.6 | 224.3 | 32.3 | 31.5 | 60.8 | +15.2% | 0.94 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2361 | 452.6 | 2684 | 427.5 | 440.2 | 2871 | +13.7% | 0.73 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p50_us | 338.9 | 48.0 | 317.6 | 50.6 | 49.3 | 93.2 | -6.3% | 0.43 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3580 | 894.5 | 3238 | 741.8 | 821.7 | 1184 | -9.6% | 0.42 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 19818 | 613.0 | 20397 | 507.6 | 562.8 | 2534 | +2.9% | 1.03 | BELOW FLOOR |
| P vs M | write_p99_us | 33402 | 4778 | 32398 | 2430 | 3791 | 1985 | -3.0% | 0.26 | BELOW FLOOR |
| P vs M | commits_per_s | 46.0 | 2.23 | 45.8 | 1.34 | 1.84 | 2.52 | -0.3% | 0.07 | BELOW FLOOR |
| P vs M | pss_mib | 43.3 | 0.04 | 43.4 | 0.02 | 0.04 | 0.10 | +0.2% | 2.64 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -7647 | 512.4 | -5678 | 345.3 | 436.9 | 9051 | -25.7% | 4.51 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 191.7 | 16.9 | 326.2 | 33.6 | 26.6 | 3.48 | +70.2% | 5.06 | **N better** |
| N vs H3 | q1_p99_us | 765.6 | 158.6 | 1083 | 272.4 | 222.9 | 180.8 | +41.4% | 1.42 | no difference |
| N vs H3 | q2_p50_us | 9701 | 330.0 | 329.6 | 31.8 | 234.5 | 857.5 | -96.6% | 40.0 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2249 | 269.1 | 3797 | 714.9 | 540.2 | 850.8 | +68.8% | 2.87 | no difference |
| N vs H3 | q6_p50_us | 4011 | 222.6 | 5598 | 147.9 | 189.0 | 235.9 | +39.6% | 8.40 | **N better** |
| N vs H3 | c2_q1_p50_us | 150.9 | 23.2 | 346.2 | 34.5 | 29.4 | 87.1 | +129.4% | 6.64 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 2552 | 1182 | 1170 | 346.2 | 870.9 | 440.8 | -54.2% | 1.59 | no difference |
| N vs H3 | c2_q2_p50_us | 10196 | 134.8 | 348.4 | 42.7 | 100.0 | 648.4 | -96.6% | 98.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 189.2 | 30.6 | 564.2 | 57.6 | 46.1 | 180.0 | +198.2% | 8.13 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 6292 | 1265 | 1687 | 292.7 | 917.9 | 1331 | -73.2% | 5.02 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 18357 | 1755 | 630.9 | 120.8 | 1244 | 478.6 | -96.6% | 14.2 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 43399 | 5353 | 1957 | 308.3 | 3791 | 4889 | -95.5% | 10.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 614.7 | 43.2 | 1487 | 117.5 | 88.5 | 189.6 | +141.9% | 9.85 | **N better** |
| N vs H3 | write_p99_us | 1244 | 446.8 | 5313 | 1180 | 892.3 | 1643 | +327.2% | 4.56 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | commits_per_s | 1468 | 99.5 | 511.5 | 86.2 | 93.1 | 201.5 | -65.2% | 10.3 | **N better** |
| N vs H3 | pss_mib | 22.9 | 2.90 | 67.3 | 0.09 | 2.05 | 0.19 | +194.5% | 21.7 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 18195 | 2069 | 12715 | 829.1 | 1576 | 9775 | -30.1% | 3.48 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 780.9 | 66.6 | 326.2 | 33.6 | 52.8 | 6.95 | -58.2% | 8.62 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2666 | 369.1 | 1083 | 272.4 | 324.4 | 826.8 | -59.4% | 4.88 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 577.6 | 43.6 | 329.6 | 31.8 | 38.1 | 65.8 | -42.9% | 6.50 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 373.8 | 61.5 | 379.4 | 47.2 | 54.8 | 138.8 | +1.5% | 0.10 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 1971 | 187.1 | 1820 | 175.4 | 181.4 | 243.8 | -7.6% | 0.83 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3833 | 197.1 | 3797 | 714.9 | 524.4 | 1039 | -1.0% | 0.07 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 4809 | 375.2 | 5598 | 147.9 | 285.2 | 1205 | +16.4% | 2.77 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 999.6 | 77.8 | 346.2 | 34.5 | 60.2 | 233.1 | -65.4% | 10.9 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 6878 | 861.7 | 1170 | 346.2 | 656.7 | 897.7 | -83.0% | 8.69 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 822.9 | 91.7 | 348.4 | 42.7 | 71.5 | 249.4 | -57.7% | 6.63 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2029 | 142.4 | 564.2 | 57.6 | 108.6 | 394.4 | -72.2% | 13.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 12638 | 3757 | 1687 | 292.7 | 2665 | 4870 | -86.7% | 4.11 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1868 | 133.3 | 630.9 | 120.8 | 127.2 | 299.5 | -66.2% | 9.73 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 10065 | 2764 | 1957 | 308.3 | 1966 | 1387 | -80.6% | 4.12 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1127 | 85.5 | 1487 | 117.5 | 102.8 | 184.0 | +31.9% | 3.50 | **P+ better** |
| P+ vs H3 | write_p99_us | 4259 | 707.9 | 5313 | 1180 | 973.1 | 1687 | +24.8% | 1.08 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 685.3 | 67.8 | 511.5 | 86.2 | 77.5 | 87.6 | -25.4% | 2.24 | no difference |
| P+ vs H3 | pss_mib | 43.3 | 0.01 | 67.3 | 0.09 | 0.06 | 0.18 | +55.3% | 376.1 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -14739 | 1014 | 12715 | 829.1 | 926.2 | 16820 | -186.3% | 29.6 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 191.7 | 16.9 | 1337 | 105.8 | 75.8 | 126.9 | +597.2% | 15.1 | **N better** |
| N vs T | q1_p99_us | 765.6 | 158.6 | 4079 | 1105 | 789.1 | 385.9 | +432.7% | 4.20 | **N better** |
| N vs T | q2_p50_us | 9701 | 330.0 | 1417 | 74.8 | 239.3 | 857.6 | -85.4% | 34.6 | **T better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 150.9 | 23.2 | 1858 | 77.2 | 57.0 | 163.2 | +1130.8% | 29.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q1_p99_us | 2552 | 1182 | 5675 | 2109 | 1709 | 477.1 | +122.3% | 1.83 | no difference |
| N vs T | c2_q2_p50_us | 10196 | 134.8 | 1908 | 67.4 | 106.6 | 664.1 | -81.3% | 77.7 | **T better** |
| N vs T | c4_q1_p50_us | 189.2 | 30.6 | 2231 | 159.3 | 114.7 | 356.4 | +1079.4% | 17.8 | **N better** |
| N vs T | c4_q1_p99_us | 6292 | 1265 | 7418 | 1623 | 1455 | 2125 | +17.9% | 0.77 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c4_q2_p50_us | 18357 | 1755 | 2382 | 227.7 | 1251 | 540.3 | -87.0% | 12.8 | **T better** |
| N vs T | c4_q2_p99_us | 43399 | 5353 | 6261 | 1455 | 3922 | 4792 | -85.6% | 9.47 | **T better** |
| N vs T | write_p50_us | 614.7 | 43.2 | 1366 | 155.6 | 114.2 | 347.7 | +122.2% | 6.58 | **N better** |
| N vs T | write_p99_us | 1244 | 446.8 | 2385 | 294.2 | 378.3 | 1695 | +91.7% | 3.02 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | commits_per_s | 1468 | 99.5 | 711.9 | 65.3 | 84.2 | 313.6 | -51.5% | 8.98 | **N better** |
| N vs T | pss_mib | 22.9 | 2.90 | 2524 | 0.03 | 2.05 | 1.63 | +10942.2% | 1220 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 18195 | 2069 | -1886 | 175.5 | 1468 | 3304 | -110.4% | 13.7 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | q1_p50_us | 780.9 | 66.6 | 1337 | 105.8 | 88.4 | 127.1 | +71.2% | 6.29 | **P+ better** |
| P+ vs T | q1_p99_us | 2666 | 369.1 | 4079 | 1105 | 823.6 | 894.3 | +53.0% | 1.72 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | q2_p50_us | 577.6 | 43.6 | 1417 | 74.8 | 61.2 | 66.7 | +145.3% | 13.7 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 999.6 | 77.8 | 1858 | 77.2 | 77.5 | 270.9 | +85.8% | 11.1 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 6878 | 861.7 | 5675 | 2109 | 1611 | 916.0 | -17.5% | 0.75 | no difference |
| P+ vs T | c2_q2_p50_us | 822.9 | 91.7 | 1908 | 67.4 | 80.5 | 287.7 | +131.8% | 13.5 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2029 | 142.4 | 2231 | 159.3 | 151.1 | 500.2 | +10.0% | 1.34 | BELOW FLOOR |
| P+ vs T | c4_q1_p99_us | 12638 | 3757 | 7418 | 1623 | 2894 | 5144 | -41.3% | 1.80 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c4_q2_p50_us | 1868 | 133.3 | 2382 | 227.7 | 186.5 | 390.6 | +27.5% | 2.75 | no difference |
| P+ vs T | c4_q2_p99_us | 10065 | 2764 | 6261 | 1455 | 2209 | 994.4 | -37.8% | 1.72 | no difference |
| P+ vs T | write_p50_us | 1127 | 85.5 | 1366 | 155.6 | 125.6 | 344.6 | +21.2% | 1.90 | BELOW FLOOR |
| P+ vs T | write_p99_us | 4259 | 707.9 | 2385 | 294.2 | 542.0 | 1737 | -44.0% | 3.46 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 685.3 | 67.8 | 711.9 | 65.3 | 66.5 | 255.8 | +3.9% | 0.40 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.01 | 2524 | 0.03 | 0.02 | 1.62 | +5722.4% | 126336 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -14739 | 1014 | -1886 | 175.5 | 727.8 | 14081 | -87.2% | 17.7 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 326.2 | 33.6 | 1337 | 105.8 | 78.5 | 126.9 | +309.8% | 12.9 | **H3 better** |
| H3 vs T | q1_p99_us | 1083 | 272.4 | 4079 | 1105 | 804.5 | 396.1 | +276.6% | 3.72 | **H3 better** |
| H3 vs T | q2_p50_us | 329.6 | 31.8 | 1417 | 74.8 | 57.5 | 48.0 | +329.9% | 18.9 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 346.2 | 34.5 | 1858 | 77.2 | 59.8 | 143.4 | +436.5% | 25.3 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 1170 | 346.2 | 5675 | 2109 | 1511 | 542.0 | +385.0% | 2.98 | no difference |
| H3 vs T | c2_q2_p50_us | 348.4 | 42.7 | 1908 | 67.4 | 56.4 | 301.7 | +447.5% | 27.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p50_us | 564.2 | 57.6 | 2231 | 159.3 | 119.8 | 397.4 | +295.5% | 13.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 1687 | 292.7 | 7418 | 1623 | 1166 | 1679 | +339.8% | 4.91 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c4_q2_p50_us | 630.9 | 120.8 | 2382 | 227.7 | 182.3 | 483.6 | +277.5% | 9.61 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p99_us | 1957 | 308.3 | 6261 | 1455 | 1052 | 1216 | +219.9% | 4.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | write_p50_us | 1487 | 117.5 | 1366 | 155.6 | 137.9 | 389.0 | -8.1% | 0.87 | BELOW FLOOR |
| H3 vs T | write_p99_us | 5313 | 1180 | 2385 | 294.2 | 860.0 | 2361 | -55.1% | 3.41 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | commits_per_s | 511.5 | 86.2 | 711.9 | 65.3 | 76.5 | 240.9 | +39.2% | 2.62 | BELOW FLOOR |
| H3 vs T | pss_mib | 67.3 | 0.09 | 2524 | 0.03 | 0.07 | 1.63 | +3650.1% | 37394 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 12715 | 829.1 | -1886 | 175.5 | 599.2 | 9334 | -114.8% | 24.4 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 191.7 | 16.9 | 1067 | 62.7 | 45.9 | 143.5 | +456.7% | 19.1 | **N better** |
| N vs H1 | q1_p99_us | 765.6 | 158.6 | 2983 | 568.1 | 417.0 | 1023 | +289.6% | 5.32 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q2_p50_us | 9701 | 330.0 | 888.6 | 90.7 | 242.0 | 860.5 | -90.8% | 36.4 | **H1 better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2249 | 269.1 | 1951 | 1397 | 1006 | 355.6 | -13.2% | 0.30 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q6_p50_us | 4011 | 222.6 | 2223 | 226.6 | 224.6 | 131.4 | -44.6% | 7.96 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 150.9 | 23.2 | 815.6 | 89.4 | 65.3 | 209.3 | +440.4% | 10.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q1_p99_us | 2552 | 1182 | 2584 | 708.5 | 974.5 | 254.6 | +1.2% | 0.03 | BELOW FLOOR |
| N vs H1 | c2_q2_p50_us | 10196 | 134.8 | 794.3 | 71.2 | 107.8 | 620.7 | -92.2% | 87.2 | **H1 better** |
| N vs H1 | c4_q1_p50_us | 189.2 | 30.6 | 895.3 | 44.6 | 38.3 | 189.5 | +373.2% | 18.4 | **N better** |
| N vs H1 | c4_q1_p99_us | 6292 | 1265 | 3154 | 526.8 | 968.8 | 1515 | -49.9% | 3.24 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c4_q2_p50_us | 18357 | 1755 | 886.5 | 54.0 | 1242 | 383.6 | -95.2% | 14.1 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 43399 | 5353 | 3575 | 1209 | 3880 | 4976 | -91.8% | 10.3 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | write_p50_us | 614.7 | 43.2 | 1876 | 75.0 | 61.2 | 57.4 | +205.1% | 20.6 | **N better** |
| N vs H1 | write_p99_us | 1244 | 446.8 | 5375 | 737.1 | 609.5 | 213.9 | +332.1% | 6.78 | **N better** |
| N vs H1 | commits_per_s | 1468 | 99.5 | 473.1 | 24.2 | 72.4 | 203.0 | -67.8% | 13.7 | **N better** |
| N vs H1 | pss_mib | 22.9 | 2.90 | 286.8 | 5.34 | 4.30 | 11.1 | +1154.9% | 61.4 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 18195 | 2069 | 27524 | 3353 | 2786 | 8274 | +51.3% | 3.35 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q1_p50_us | 780.9 | 66.6 | 1067 | 62.7 | 64.7 | 143.7 | +36.7% | 4.43 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2666 | 369.1 | 2983 | 568.1 | 479.0 | 1303 | +11.9% | 0.66 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | q2_p50_us | 577.6 | 43.6 | 888.6 | 90.7 | 71.1 | 97.0 | +53.8% | 4.37 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 373.8 | 61.5 | 947.9 | 112.0 | 90.3 | 209.6 | +153.6% | 6.36 | **P+ better** |
| P+ vs H1 | q4_p50_us | 1971 | 187.1 | 4317 | 666.5 | 489.5 | 697.9 | +119.0% | 4.79 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3833 | 197.1 | 1951 | 1397 | 997.4 | 694.3 | -49.1% | 1.89 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q6_p50_us | 4809 | 375.2 | 2223 | 226.6 | 309.9 | 1189 | -53.8% | 8.34 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 999.6 | 77.8 | 815.6 | 89.4 | 83.8 | 300.9 | -18.4% | 2.20 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 6878 | 861.7 | 2584 | 708.5 | 788.8 | 822.4 | -62.4% | 5.44 | **H1 better** |
| P+ vs H1 | c2_q2_p50_us | 822.9 | 91.7 | 794.3 | 71.2 | 82.1 | 164.3 | -3.5% | 0.35 | BELOW FLOOR |
| P+ vs H1 | c4_q1_p50_us | 2029 | 142.4 | 895.3 | 44.6 | 105.5 | 398.8 | -55.9% | 10.7 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 12638 | 3757 | 3154 | 526.8 | 2683 | 4923 | -75.0% | 3.54 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c4_q2_p50_us | 1868 | 133.3 | 886.5 | 54.0 | 101.7 | 88.3 | -52.6% | 9.66 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 10065 | 2764 | 3575 | 1209 | 2133 | 1669 | -64.5% | 3.04 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | write_p50_us | 1127 | 85.5 | 1876 | 75.0 | 80.4 | 34.2 | +66.4% | 9.31 | **P+ better** |
| P+ vs H1 | write_p99_us | 4259 | 707.9 | 5375 | 737.1 | 722.6 | 436.0 | +26.2% | 1.54 | no difference |
| P+ vs H1 | commits_per_s | 685.3 | 67.8 | 473.1 | 24.2 | 50.9 | 91.2 | -31.0% | 4.17 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.01 | 286.8 | 5.34 | 3.78 | 11.1 | +561.7% | 64.5 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -14739 | 1014 | 27524 | 3353 | 2477 | 15994 | -286.7% | 17.1 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 191.7 | 16.9 | 140.5 | 15.2 | 16.1 | 26.1 | -26.7% | 3.19 | **H2 better** |
| N vs H2 | q1_p99_us | 765.6 | 158.6 | 750.5 | 258.9 | 214.7 | 205.6 | -2.0% | 0.07 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 9701 | 330.0 | 239.4 | 12.8 | 233.6 | 857.2 | -97.5% | 40.5 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2249 | 269.1 | 176.8 | 35.5 | 191.9 | 263.7 | -92.1% | 10.8 | **H2 better** |
| N vs H2 | q6_p50_us | 4011 | 222.6 | 999.3 | 129.0 | 181.9 | 80.4 | -75.1% | 16.6 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 150.9 | 23.2 | 163.8 | 60.2 | 45.6 | 126.6 | +8.5% | 0.28 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q1_p99_us | 2552 | 1182 | 2311 | 523.9 | 914.2 | 387.9 | -9.5% | 0.26 | BELOW FLOOR |
| N vs H2 | c2_q2_p50_us | 10196 | 134.8 | 322.0 | 64.1 | 105.6 | 628.8 | -96.8% | 93.5 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p50_us | 189.2 | 30.6 | 210.2 | 29.7 | 30.2 | 30.0 | +11.1% | 0.69 | BELOW FLOOR |
| N vs H2 | c4_q1_p99_us | 6292 | 1265 | 2562 | 449.9 | 949.2 | 1494 | -59.3% | 3.93 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 18357 | 1755 | 350.8 | 22.8 | 1241 | 381.4 | -98.1% | 14.5 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 43399 | 5353 | 3165 | 339.4 | 3793 | 4951 | -92.7% | 10.6 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 614.7 | 43.2 | 1595 | 106.5 | 81.3 | 138.4 | +159.5% | 12.1 | **N better** |
| N vs H2 | write_p99_us | 1244 | 446.8 | 5579 | 481.6 | 464.5 | 1443 | +348.5% | 9.33 | **N better** |
| N vs H2 | commits_per_s | 1468 | 99.5 | 499.6 | 28.9 | 73.3 | 206.1 | -66.0% | 13.2 | **N better** |
| N vs H2 | pss_mib | 22.9 | 2.90 | 43.4 | 0.02 | 2.05 | 0.09 | +90.0% | 10.0 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 18195 | 2069 | -2914 | 208.2 | 1470 | 4213 | -116.0% | 14.4 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p50_us | 780.9 | 66.6 | 140.5 | 15.2 | 48.3 | 26.8 | -82.0% | 13.2 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2666 | 369.1 | 750.5 | 258.9 | 318.8 | 832.6 | -71.9% | 6.01 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q2_p50_us | 577.6 | 43.6 | 239.4 | 12.8 | 32.1 | 61.2 | -58.6% | 10.5 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 373.8 | 61.5 | 248.9 | 36.3 | 50.5 | 35.6 | -33.4% | 2.47 | no difference |
| P+ vs H2 | q4_p50_us | 1971 | 187.1 | 288.6 | 35.5 | 134.6 | 167.4 | -85.4% | 12.5 | **H2 better** |
| P+ vs H2 | q5_p50_us | 3833 | 197.1 | 176.8 | 35.5 | 141.6 | 652.0 | -95.4% | 25.8 | **H2 better** |
| P+ vs H2 | q6_p50_us | 4809 | 375.2 | 999.3 | 129.0 | 280.5 | 1185 | -79.2% | 13.6 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 999.6 | 77.8 | 163.8 | 60.2 | 69.5 | 250.5 | -83.6% | 12.0 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q1_p99_us | 6878 | 861.7 | 2311 | 523.9 | 713.1 | 872.9 | -66.4% | 6.40 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 822.9 | 91.7 | 322.0 | 64.1 | 79.1 | 192.8 | -60.9% | 6.33 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p50_us | 2029 | 142.4 | 210.2 | 29.7 | 102.8 | 352.2 | -89.6% | 17.7 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 12638 | 3757 | 2562 | 449.9 | 2676 | 4917 | -79.7% | 3.77 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1868 | 133.3 | 350.8 | 22.8 | 95.6 | 78.1 | -81.2% | 15.9 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 10065 | 2764 | 3165 | 339.4 | 1969 | 1591 | -68.6% | 3.50 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | write_p50_us | 1127 | 85.5 | 1595 | 106.5 | 96.6 | 130.5 | +41.5% | 4.85 | **P+ better** |
| P+ vs H2 | write_p99_us | 4259 | 707.9 | 5579 | 481.6 | 605.4 | 1493 | +31.0% | 2.18 | BELOW FLOOR |
| P+ vs H2 | commits_per_s | 685.3 | 67.8 | 499.6 | 28.9 | 52.1 | 97.7 | -27.1% | 3.57 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.01 | 43.4 | 0.02 | 0.02 | 0.04 | +0.2% | 4.97 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -14739 | 1014 | -2914 | 208.2 | 732.1 | 14322 | -80.2% | 16.2 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 178.4 | 5.47 | 749.8 | 70.4 | 50.0 | 152.2 | +320.2% | 11.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 549.6 | 62.0 | 2863 | 550.0 | 391.3 | 2807 | +420.9% | 5.91 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 9696 | 642.7 | 560.6 | 62.2 | 456.6 | 1569 | -94.2% | 20.0 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2225 | 314.8 | 3709 | 191.0 | 260.4 | 1195 | +66.7% | 5.70 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q6_p50_us | 4045 | 163.0 | 5676 | 869.2 | 625.3 | 926.4 | +40.3% | 2.61 | no difference |
| N vs P+ | c2_q1_p50_us | 139.8 | 34.0 | 1132 | 83.6 | 63.8 | 29.5 | +710.3% | 15.6 | **N better** |
| N vs P+ | c2_q1_p99_us | 2003 | 905.3 | 7253 | 1120 | 1018 | 2751 | +262.1% | 5.15 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 10019 | 374.2 | 926.5 | 113.2 | 276.4 | 2737 | -90.8% | 32.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 170.8 | 19.3 | 2078 | 112.1 | 80.4 | 84.6 | +1117.1% | 23.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 7679 | 542.1 | 12331 | 2991 | 2150 | 2296 | +60.6% | 2.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 13875 | 2506 | 1766 | 88.2 | 1773 | 2702 | -87.3% | 6.83 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 42100 | 3346 | 12763 | 2703 | 3042 | 6321 | -69.7% | 9.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 611.0 | 65.7 | 1123 | 105.3 | 87.7 | 142.9 | +83.8% | 5.84 | **N better** |
| N vs P+ | write_p99_us | 1072 | 264.9 | 3983 | 392.6 | 334.8 | 1291 | +271.4% | 8.69 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1554 | 197.9 | 712.2 | 65.6 | 147.4 | 400.9 | -54.2% | 5.71 | **N better** |
| N vs P+ | pss_mib | 25.2 | 2.95 | 43.4 | 0.01 | 2.09 | 0.02 | +72.5% | 8.75 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 19694 | 703.1 | -1996 | 129.8 | 505.6 | 9210 | -110.1% | 42.9 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 178.4 | 5.47 | 148.1 | 18.5 | 13.6 | 121.4 | -17.0% | 2.22 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q1_p99_us | 549.6 | 62.0 | 857.9 | 255.4 | 185.8 | 496.3 | +56.1% | 1.66 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 9696 | 642.7 | 231.3 | 9.82 | 454.5 | 1570 | -97.6% | 20.8 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2225 | 314.8 | 218.2 | 27.0 | 223.4 | 1151 | -90.2% | 8.98 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q6_p50_us | 4045 | 163.0 | 897.7 | 89.3 | 131.4 | 918.5 | -77.8% | 23.9 | **P better** |
| N vs P | c2_q1_p50_us | 139.8 | 34.0 | 183.2 | 46.3 | 40.6 | 25.7 | +31.0% | 1.07 | no difference |
| N vs P | c2_q1_p99_us | 2003 | 905.3 | 2075 | 523.4 | 739.4 | 2301 | +3.6% | 0.10 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 10019 | 374.2 | 277.5 | 31.6 | 265.5 | 2736 | -97.2% | 36.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 170.8 | 19.3 | 210.8 | 16.5 | 18.0 | 107.1 | +23.4% | 2.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 7679 | 542.1 | 2695 | 405.2 | 478.6 | 2063 | -64.9% | 10.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 13875 | 2506 | 339.5 | 35.8 | 1772 | 2687 | -97.6% | 7.64 | **P better** |
| N vs P | c4_q2_p99_us | 42100 | 3346 | 3268 | 729.2 | 2421 | 5066 | -92.2% | 16.0 | **P better** |
| N vs P | write_p50_us | 611.0 | 65.7 | 20274 | 1352 | 957.3 | 1325 | +3218.1% | 20.5 | **N better** |
| N vs P | write_p99_us | 1072 | 264.9 | 33728 | 748.3 | 561.3 | 1899 | +3045.2% | 58.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1554 | 197.9 | 43.5 | 3.65 | 139.9 | 350.7 | -97.2% | 10.8 | **N better** |
| N vs P | pss_mib | 25.2 | 2.95 | 43.4 | 0.03 | 2.09 | 0.07 | +72.3% | 8.72 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 19694 | 703.1 | -2136 | 141.6 | 507.1 | 9222 | -110.8% | 43.0 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 749.8 | 70.4 | 148.1 | 18.5 | 51.5 | 91.9 | -80.2% | 11.7 | **P better** |
| P+ vs P | q1_p99_us | 2863 | 550.0 | 857.9 | 255.4 | 428.8 | 2776 | -70.0% | 4.68 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 560.6 | 62.2 | 231.3 | 9.82 | 44.5 | 46.9 | -58.7% | 7.39 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 313.5 | 27.1 | 266.1 | 44.2 | 36.7 | 115.6 | -15.1% | 1.29 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q4_p50_us | 1855 | 226.7 | 301.0 | 47.5 | 163.8 | 114.1 | -83.8% | 9.49 | **P better** |
| P+ vs P | q5_p50_us | 3709 | 191.0 | 218.2 | 27.0 | 136.4 | 326.3 | -94.1% | 25.6 | **P better** |
| P+ vs P | q6_p50_us | 5676 | 869.2 | 897.7 | 89.3 | 617.9 | 240.1 | -84.2% | 7.73 | **P better** |
| P+ vs P | c2_q1_p50_us | 1132 | 83.6 | 183.2 | 46.3 | 67.5 | 15.3 | -83.8% | 14.1 | **P better** |
| P+ vs P | c2_q1_p99_us | 7253 | 1120 | 2075 | 523.4 | 874.4 | 1866 | -71.4% | 5.92 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 926.5 | 113.2 | 277.5 | 31.6 | 83.1 | 71.9 | -70.0% | 7.81 | **P better** |
| P+ vs P | c4_q1_p50_us | 2078 | 112.1 | 210.8 | 16.5 | 80.1 | 135.3 | -89.9% | 23.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 12331 | 2991 | 2695 | 405.2 | 2134 | 1014 | -78.1% | 4.51 | **P better** |
| P+ vs P | c4_q2_p50_us | 1766 | 88.2 | 339.5 | 35.8 | 67.3 | 283.2 | -80.8% | 21.2 | **P better** |
| P+ vs P | c4_q2_p99_us | 12763 | 2703 | 3268 | 729.2 | 1980 | 3812 | -74.4% | 4.80 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 1123 | 105.3 | 20274 | 1352 | 959.1 | 1320 | +1705.1% | 20.0 | **P+ better** |
| P+ vs P | write_p99_us | 3983 | 392.6 | 33728 | 748.3 | 597.5 | 2143 | +746.8% | 49.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 712.2 | 65.6 | 43.5 | 3.65 | 46.4 | 194.2 | -93.9% | 14.4 | **P+ better** |
| P+ vs P | pss_mib | 43.4 | 0.01 | 43.4 | 0.03 | 0.02 | 0.08 | -0.1% | 2.28 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -1996 | 129.8 | -2136 | 141.6 | 135.9 | 2380 | +7.0% | 1.03 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 148.1 | 18.5 | 144.5 | 19.3 | 18.9 | 10.9 | -2.5% | 0.19 | BELOW FLOOR |
| P vs M | q1_p99_us | 857.9 | 255.4 | 1133 | 351.4 | 307.2 | 479.7 | +32.1% | 0.90 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 231.3 | 9.82 | 240.5 | 17.9 | 14.5 | 59.1 | +4.0% | 0.64 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 266.1 | 44.2 | 260.3 | 25.9 | 36.2 | 102.4 | -2.2% | 0.16 | BELOW FLOOR |
| P vs M | q4_p50_us | 301.0 | 47.5 | 276.5 | 16.9 | 35.6 | 114.7 | -8.1% | 0.69 | BELOW FLOOR |
| P vs M | q5_p50_us | 218.2 | 27.0 | 194.4 | 14.8 | 21.8 | 51.1 | -10.9% | 1.09 | BELOW FLOOR |
| P vs M | q6_p50_us | 897.7 | 89.3 | 808.1 | 63.5 | 77.5 | 174.4 | -10.0% | 1.16 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 183.2 | 46.3 | 154.0 | 41.7 | 44.0 | 102.3 | -15.9% | 0.66 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 2075 | 523.4 | 1745 | 778.1 | 663.1 | 913.2 | -15.9% | 0.50 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 277.5 | 31.6 | 233.2 | 45.3 | 39.0 | 75.0 | -16.0% | 1.14 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 210.8 | 16.5 | 228.0 | 24.9 | 21.1 | 106.8 | +8.2% | 0.82 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2695 | 405.2 | 3431 | 722.0 | 585.4 | 278.3 | +27.3% | 1.26 | no difference |
| P vs M | c4_q2_p50_us | 339.5 | 35.8 | 344.8 | 68.4 | 54.6 | 142.0 | +1.5% | 0.10 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p99_us | 3268 | 729.2 | 3528 | 789.5 | 759.9 | 1017 | +8.0% | 0.34 | BELOW FLOOR |
| P vs M | write_p50_us | 20274 | 1352 | 19957 | 714.5 | 1081 | 1776 | -1.6% | 0.29 | BELOW FLOOR |
| P vs M | write_p99_us | 33728 | 748.3 | 27764 | 3294 | 2388 | 1951 | -17.7% | 2.50 | no difference |
| P vs M | commits_per_s | 43.5 | 3.65 | 46.3 | 1.99 | 2.94 | 3.17 | +6.3% | 0.94 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.03 | 43.3 | 0.02 | 0.03 | 0.30 | -0.1% | 1.64 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2136 | 141.6 | -2061 | 99.2 | 122.3 | 7006 | -3.5% | 0.61 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 178.4 | 5.47 | 322.2 | 37.0 | 26.4 | 159.9 | +80.6% | 5.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 549.6 | 62.0 | 911.8 | 173.8 | 130.5 | 1012 | +65.9% | 2.78 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 9696 | 642.7 | 320.8 | 26.7 | 454.8 | 1572 | -96.7% | 20.6 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2225 | 314.8 | 3768 | 668.9 | 522.7 | 1878 | +69.4% | 2.95 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q6_p50_us | 4045 | 163.0 | 5803 | 517.3 | 383.5 | 1280 | +43.5% | 4.59 | **N better** |
| N vs H3 | c2_q1_p50_us | 139.8 | 34.0 | 301.1 | 45.1 | 40.0 | 161.7 | +115.4% | 4.04 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 2003 | 905.3 | 1396 | 250.1 | 664.1 | 2371 | -30.3% | 0.91 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 10019 | 374.2 | 367.4 | 68.3 | 268.9 | 2736 | -96.3% | 35.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 170.8 | 19.3 | 552.9 | 74.9 | 54.7 | 158.3 | +223.8% | 6.98 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 7679 | 542.1 | 1880 | 419.0 | 484.5 | 2663 | -75.5% | 12.0 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 13875 | 2506 | 630.5 | 101.1 | 1773 | 2687 | -95.5% | 7.47 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 42100 | 3346 | 1948 | 360.2 | 2379 | 5055 | -95.4% | 16.9 | **H3 better** |
| N vs H3 | write_p50_us | 611.0 | 65.7 | 1541 | 65.5 | 65.6 | 342.9 | +152.2% | 14.2 | **N better** |
| N vs H3 | write_p99_us | 1072 | 264.9 | 4654 | 761.5 | 570.1 | 627.3 | +334.0% | 6.28 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1554 | 197.9 | 565.9 | 74.4 | 149.5 | 356.6 | -63.6% | 6.61 | **N better** |
| N vs H3 | pss_mib | 25.2 | 2.95 | 66.6 | 0.09 | 2.09 | 0.09 | +164.7% | 19.9 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 19694 | 703.1 | 12724 | 568.6 | 639.4 | 14187 | -35.4% | 10.9 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 749.8 | 70.4 | 322.2 | 37.0 | 56.2 | 138.9 | -57.0% | 7.60 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2863 | 550.0 | 911.8 | 173.8 | 407.8 | 2913 | -68.1% | 4.78 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 560.6 | 62.2 | 320.8 | 26.7 | 47.9 | 101.9 | -42.8% | 5.01 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 313.5 | 27.1 | 321.6 | 64.1 | 49.2 | 128.0 | +2.6% | 0.17 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | 1855 | 226.7 | 1824 | 73.9 | 168.6 | 1105 | -1.6% | 0.18 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q5_p50_us | 3709 | 191.0 | 3768 | 668.9 | 491.8 | 1519 | +1.6% | 0.12 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q6_p50_us | 5676 | 869.2 | 5803 | 517.3 | 715.2 | 923.6 | +2.2% | 0.18 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 1132 | 83.6 | 301.1 | 45.1 | 67.1 | 160.4 | -73.4% | 12.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 7253 | 1120 | 1396 | 250.1 | 811.7 | 1952 | -80.8% | 7.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 926.5 | 113.2 | 367.4 | 68.3 | 93.5 | 60.3 | -60.3% | 5.98 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2078 | 112.1 | 552.9 | 74.9 | 95.3 | 178.6 | -73.4% | 16.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 12331 | 2991 | 1880 | 419.0 | 2136 | 1966 | -84.8% | 4.89 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1766 | 88.2 | 630.5 | 101.1 | 94.9 | 286.6 | -64.3% | 12.0 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 12763 | 2703 | 1948 | 360.2 | 1928 | 3798 | -84.7% | 5.61 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1123 | 105.3 | 1541 | 65.5 | 87.7 | 325.7 | +37.2% | 4.76 | **P+ better** |
| P+ vs H3 | write_p99_us | 3983 | 392.6 | 4654 | 761.5 | 605.8 | 1174 | +16.8% | 1.11 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 712.2 | 65.6 | 565.9 | 74.4 | 70.1 | 204.7 | -20.6% | 2.09 | BELOW FLOOR |
| P+ vs H3 | pss_mib | 43.4 | 0.01 | 66.6 | 0.09 | 0.07 | 0.09 | +53.4% | 351.5 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -1996 | 129.8 | 12724 | 568.6 | 412.4 | 11040 | -737.4% | 35.7 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 178.4 | 5.47 | 1306 | 66.2 | 47.0 | 434.3 | +632.0% | 24.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q1_p99_us | 549.6 | 62.0 | 3617 | 658.9 | 468.0 | 2212 | +558.2% | 6.56 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | q2_p50_us | 9696 | 642.7 | 1379 | 145.5 | 465.9 | 1576 | -85.8% | 17.9 | **T better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 139.8 | 34.0 | 1860 | 62.8 | 50.5 | 262.9 | +1230.9% | 34.1 | **N better** |
| N vs T | c2_q1_p99_us | 2003 | 905.3 | 3760 | 614.5 | 773.6 | 8631 | +87.7% | 2.27 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c2_q2_p50_us | 10019 | 374.2 | 1965 | 103.0 | 274.4 | 2745 | -80.4% | 29.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p50_us | 170.8 | 19.3 | 2298 | 179.8 | 127.9 | 410.0 | +1245.9% | 16.6 | **N better** |
| N vs T | c4_q1_p99_us | 7679 | 542.1 | 7839 | 2578 | 1863 | 2362 | +2.1% | 0.09 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p50_us | 13875 | 2506 | 2329 | 147.1 | 1775 | 2701 | -83.2% | 6.51 | **T better** |
| N vs T | c4_q2_p99_us | 42100 | 3346 | 7987 | 1435 | 2574 | 5829 | -81.0% | 13.3 | **T better** |
| N vs T | write_p50_us | 611.0 | 65.7 | 1318 | 137.7 | 107.9 | 447.4 | +115.7% | 6.56 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p99_us | 1072 | 264.9 | 2623 | 416.8 | 349.2 | 665.8 | +144.6% | 4.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | commits_per_s | 1554 | 197.9 | 700.6 | 61.3 | 146.5 | 430.8 | -54.9% | 5.83 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | pss_mib | 25.2 | 2.95 | 2525 | 0.01 | 2.09 | 0.11 | +9932.1% | 1199 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 19694 | 703.1 | -2168 | 149.8 | 508.3 | 9201 | -111.0% | 43.0 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 749.8 | 70.4 | 1306 | 66.2 | 68.3 | 427.0 | +74.2% | 8.14 | **P+ better** |
| P+ vs T | q1_p99_us | 2863 | 550.0 | 3617 | 658.9 | 606.9 | 3514 | +26.4% | 1.24 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | q2_p50_us | 560.6 | 62.2 | 1379 | 145.5 | 111.9 | 150.6 | +145.9% | 7.31 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 1132 | 83.6 | 1860 | 62.8 | 73.9 | 262.1 | +64.2% | 9.85 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 7253 | 1120 | 3760 | 614.5 | 903.5 | 8525 | -48.2% | 3.87 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c2_q2_p50_us | 926.5 | 113.2 | 1965 | 103.0 | 108.2 | 228.6 | +112.0% | 9.59 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2078 | 112.1 | 2298 | 179.8 | 149.8 | 418.2 | +10.6% | 1.47 | BELOW FLOOR |
| P+ vs T | c4_q1_p99_us | 12331 | 2991 | 7839 | 2578 | 2792 | 1534 | -36.4% | 1.61 | no difference |
| P+ vs T | c4_q2_p50_us | 1766 | 88.2 | 2329 | 147.1 | 121.3 | 391.7 | +31.9% | 4.64 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 12763 | 2703 | 7987 | 1435 | 2164 | 4780 | -37.4% | 2.21 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | write_p50_us | 1123 | 105.3 | 1318 | 137.7 | 122.5 | 434.4 | +17.4% | 1.59 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | write_p99_us | 3983 | 392.6 | 2623 | 416.8 | 404.9 | 1195 | -34.2% | 3.36 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | commits_per_s | 712.2 | 65.6 | 700.6 | 61.3 | 63.5 | 316.8 | -1.6% | 0.18 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | pss_mib | 43.4 | 0.01 | 2525 | 0.01 | 0.01 | 0.11 | +5716.2% | 205345 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -1996 | 129.8 | -2168 | 149.8 | 140.2 | 2296 | +8.6% | 1.23 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 322.2 | 37.0 | 1306 | 66.2 | 53.6 | 429.8 | +305.3% | 18.4 | **H3 better** |
| H3 vs T | q1_p99_us | 911.8 | 173.8 | 3617 | 658.9 | 481.8 | 2345 | +296.7% | 5.62 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | q2_p50_us | 320.8 | 26.7 | 1379 | 145.5 | 104.6 | 180.8 | +329.7% | 10.1 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 301.1 | 45.1 | 1860 | 62.8 | 54.6 | 306.5 | +517.8% | 28.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q1_p99_us | 1396 | 250.1 | 3760 | 614.5 | 469.1 | 8410 | +169.4% | 5.04 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c2_q2_p50_us | 367.4 | 68.3 | 1965 | 103.0 | 87.4 | 220.8 | +434.7% | 18.3 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 552.9 | 74.9 | 2298 | 179.8 | 137.7 | 439.1 | +315.7% | 12.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 1880 | 419.0 | 7839 | 2578 | 1847 | 2042 | +316.9% | 3.23 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p50_us | 630.5 | 101.1 | 2329 | 147.1 | 126.2 | 274.1 | +269.3% | 13.5 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 1948 | 360.2 | 7987 | 1435 | 1046 | 2907 | +310.1% | 5.77 | **H3 better** |
| H3 vs T | write_p50_us | 1541 | 65.5 | 1318 | 137.7 | 107.8 | 534.6 | -14.5% | 2.07 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | write_p99_us | 4654 | 761.5 | 2623 | 416.8 | 613.8 | 394.0 | -43.6% | 3.31 | **T better** |
| H3 vs T | commits_per_s | 565.9 | 74.4 | 700.6 | 61.3 | 68.2 | 258.4 | +23.8% | 1.98 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | pss_mib | 66.6 | 0.09 | 2525 | 0.01 | 0.07 | 0.14 | +3690.4% | 37073 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 12724 | 568.6 | -2168 | 149.8 | 415.8 | 11033 | -117.0% | 35.8 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 178.4 | 5.47 | 953.8 | 33.2 | 23.8 | 128.5 | +434.6% | 32.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q1_p99_us | 549.6 | 62.0 | 2682 | 419.0 | 299.5 | 1197 | +388.1% | 7.12 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | q2_p50_us | 9696 | 642.7 | 823.2 | 85.4 | 458.4 | 1571 | -91.5% | 19.4 | **H1 better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2225 | 314.8 | 4863 | 3061 | 2176 | 1152 | +118.6% | 1.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q6_p50_us | 4045 | 163.0 | 1893 | 71.3 | 125.8 | 944.4 | -53.2% | 17.1 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 139.8 | 34.0 | 935.6 | 67.1 | 53.2 | 98.2 | +569.4% | 15.0 | **N better** |
| N vs H1 | c2_q1_p99_us | 2003 | 905.3 | 3199 | 897.7 | 901.5 | 3047 | +59.7% | 1.33 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | c2_q2_p50_us | 10019 | 374.2 | 947.0 | 61.6 | 268.1 | 2741 | -90.5% | 33.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p50_us | 170.8 | 19.3 | 881.5 | 93.6 | 67.6 | 334.0 | +416.2% | 10.5 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c4_q1_p99_us | 7679 | 542.1 | 2786 | 523.3 | 532.8 | 2062 | -63.7% | 9.18 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p50_us | 13875 | 2506 | 890.6 | 24.0 | 1772 | 2702 | -93.6% | 7.33 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 42100 | 3346 | 3338 | 795.7 | 2432 | 5109 | -92.1% | 15.9 | **H1 better** |
| N vs H1 | write_p50_us | 611.0 | 65.7 | 1864 | 122.5 | 98.3 | 163.5 | +205.1% | 12.8 | **N better** |
| N vs H1 | write_p99_us | 1072 | 264.9 | 5165 | 424.1 | 353.6 | 1005 | +381.7% | 11.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | commits_per_s | 1554 | 197.9 | 483.9 | 6.83 | 140.0 | 350.9 | -68.9% | 7.64 | **N better** |
| N vs H1 | pss_mib | 25.2 | 2.95 | 285.4 | 3.53 | 3.25 | 5.54 | +1033.9% | 80.0 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 19694 | 703.1 | 32426 | 1925 | 1449 | 11220 | +64.6% | 8.79 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H1 | q1_p50_us | 749.8 | 70.4 | 953.8 | 33.2 | 55.1 | 101.1 | +27.2% | 3.71 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2863 | 550.0 | 2682 | 419.0 | 488.9 | 2982 | -6.3% | 0.37 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | q2_p50_us | 560.6 | 62.2 | 823.2 | 85.4 | 74.7 | 75.4 | +46.9% | 3.51 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 313.5 | 27.1 | 903.0 | 124.7 | 90.2 | 163.4 | +188.0% | 6.53 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q4_p50_us | 1855 | 226.7 | 3766 | 360.9 | 301.3 | 1006 | +103.0% | 6.34 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3709 | 191.0 | 4863 | 3061 | 2169 | 330.5 | +31.1% | 0.53 | no difference |
| P+ vs H1 | q6_p50_us | 5676 | 869.2 | 1893 | 71.3 | 616.7 | 325.4 | -66.6% | 6.13 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 1132 | 83.6 | 935.6 | 67.1 | 75.8 | 96.0 | -17.4% | 2.60 | no difference |
| P+ vs H1 | c2_q1_p99_us | 7253 | 1120 | 3199 | 897.7 | 1015 | 2733 | -55.9% | 3.99 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | c2_q2_p50_us | 926.5 | 113.2 | 947.0 | 61.6 | 91.1 | 174.4 | +2.2% | 0.22 | BELOW FLOOR |
| P+ vs H1 | c4_q1_p50_us | 2078 | 112.1 | 881.5 | 93.6 | 103.3 | 344.2 | -57.6% | 11.6 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | c4_q1_p99_us | 12331 | 2991 | 2786 | 523.3 | 2147 | 1012 | -77.4% | 4.45 | **H1 better** |
| P+ vs H1 | c4_q2_p50_us | 1766 | 88.2 | 890.6 | 24.0 | 64.7 | 403.8 | -49.6% | 13.5 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 12763 | 2703 | 3338 | 795.7 | 1993 | 3870 | -73.8% | 4.73 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | write_p50_us | 1123 | 105.3 | 1864 | 122.5 | 114.2 | 123.6 | +66.0% | 6.49 | **P+ better** |
| P+ vs H1 | write_p99_us | 3983 | 392.6 | 5165 | 424.1 | 408.7 | 1412 | +29.7% | 2.89 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | commits_per_s | 712.2 | 65.6 | 483.9 | 6.83 | 46.6 | 194.6 | -32.1% | 4.90 | **P+ better** |
| P+ vs H1 | pss_mib | 43.4 | 0.01 | 285.4 | 3.53 | 2.50 | 5.54 | +557.4% | 97.0 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -1996 | 129.8 | 32426 | 1925 | 1364 | 6819 | -1724.5% | 25.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 178.4 | 5.47 | 151.3 | 13.4 | 10.3 | 121.4 | -15.2% | 2.65 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q1_p99_us | 549.6 | 62.0 | 1143 | 362.5 | 260.0 | 580.0 | +107.9% | 2.28 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | q2_p50_us | 9696 | 642.7 | 239.1 | 23.9 | 454.7 | 1570 | -97.5% | 20.8 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2225 | 314.8 | 174.6 | 25.1 | 223.3 | 1151 | -92.2% | 9.18 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q6_p50_us | 4045 | 163.0 | 851.0 | 138.8 | 151.4 | 909.1 | -79.0% | 21.1 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 139.8 | 34.0 | 182.3 | 29.6 | 31.9 | 33.7 | +30.4% | 1.33 | no difference |
| N vs H2 | c2_q1_p99_us | 2003 | 905.3 | 1938 | 596.0 | 766.4 | 2263 | -3.3% | 0.09 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q2_p50_us | 10019 | 374.2 | 276.1 | 69.6 | 269.1 | 2738 | -97.2% | 36.2 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q1_p50_us | 170.8 | 19.3 | 204.1 | 30.7 | 25.6 | 12.2 | +19.5% | 1.30 | no difference |
| N vs H2 | c4_q1_p99_us | 7679 | 542.1 | 2727 | 361.6 | 460.8 | 2827 | -64.5% | 10.7 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q2_p50_us | 13875 | 2506 | 331.4 | 29.8 | 1772 | 2687 | -97.6% | 7.64 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 42100 | 3346 | 2838 | 381.7 | 2381 | 5093 | -93.3% | 16.5 | **H2 better** |
| N vs H2 | write_p50_us | 611.0 | 65.7 | 1784 | 97.1 | 82.9 | 140.3 | +192.0% | 14.2 | **N better** |
| N vs H2 | write_p99_us | 1072 | 264.9 | 5941 | 764.1 | 571.8 | 1136 | +454.0% | 8.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1554 | 197.9 | 485.4 | 26.2 | 141.1 | 355.8 | -68.8% | 7.57 | **N better** |
| N vs H2 | pss_mib | 25.2 | 2.95 | 43.4 | 0.01 | 2.09 | 0.00 | +72.6% | 8.76 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 19694 | 703.1 | -2212 | 170.3 | 511.5 | 9258 | -111.2% | 42.8 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 749.8 | 70.4 | 151.3 | 13.4 | 50.7 | 91.9 | -79.8% | 11.8 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2863 | 550.0 | 1143 | 362.5 | 465.7 | 2792 | -60.1% | 3.69 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q2_p50_us | 560.6 | 62.2 | 239.1 | 23.9 | 47.1 | 47.1 | -57.4% | 6.82 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 313.5 | 27.1 | 287.6 | 38.8 | 33.5 | 103.9 | -8.3% | 0.77 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q4_p50_us | 1855 | 226.7 | 386.9 | 59.2 | 165.7 | 90.0 | -79.1% | 8.86 | **H2 better** |
| P+ vs H2 | q5_p50_us | 3709 | 191.0 | 174.6 | 25.1 | 136.2 | 325.3 | -95.3% | 26.0 | **H2 better** |
| P+ vs H2 | q6_p50_us | 5676 | 869.2 | 851.0 | 138.8 | 622.4 | 200.9 | -85.0% | 7.75 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 1132 | 83.6 | 182.3 | 29.6 | 62.7 | 26.5 | -83.9% | 15.2 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 7253 | 1120 | 1938 | 596.0 | 897.3 | 1819 | -73.3% | 5.92 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 926.5 | 113.2 | 276.1 | 69.6 | 94.0 | 107.8 | -70.2% | 6.92 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p50_us | 2078 | 112.1 | 204.1 | 30.7 | 82.2 | 83.7 | -90.2% | 22.8 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 12331 | 2991 | 2727 | 361.6 | 2130 | 2183 | -77.9% | 4.51 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p50_us | 1766 | 88.2 | 331.4 | 29.8 | 65.9 | 285.0 | -81.2% | 21.8 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 12763 | 2703 | 2838 | 381.7 | 1930 | 3848 | -77.8% | 5.14 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 1123 | 105.3 | 1784 | 97.1 | 101.3 | 90.6 | +58.8% | 6.53 | **P+ better** |
| P+ vs H2 | write_p99_us | 3983 | 392.6 | 5941 | 764.1 | 607.4 | 1508 | +49.2% | 3.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 712.2 | 65.6 | 485.4 | 26.2 | 49.9 | 203.2 | -31.9% | 4.54 | **P+ better** |
| P+ vs H2 | pss_mib | 43.4 | 0.01 | 43.4 | 0.01 | 0.01 | 0.02 | +0.1% | 2.49 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -1996 | 129.8 | -2212 | 170.3 | 151.4 | 2516 | +10.8% | 1.42 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 168.8 | 15.3 | 689.8 | 32.3 | 25.3 | 36.4 | +308.7% | 20.6 | **N better** |
| N vs P+ | q1_p99_us | 493.2 | 62.9 | 2118 | 529.5 | 377.0 | 1749 | +329.3% | 4.31 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 9386 | 507.2 | 536.9 | 25.4 | 359.1 | 1218 | -94.3% | 24.6 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2079 | 304.1 | 3828 | 775.4 | 589.0 | 310.7 | +84.1% | 2.97 | no difference |
| N vs P+ | q6_p50_us | 4018 | 264.9 | 5095 | 429.8 | 357.0 | 892.1 | +26.8% | 3.02 | **N better** |
| N vs P+ | c2_q1_p50_us | 109.5 | 24.2 | 1080 | 88.9 | 65.1 | 61.5 | +886.3% | 14.9 | **N better** |
| N vs P+ | c2_q1_p99_us | 865.8 | 250.7 | 5837 | 932.6 | 682.8 | 2440 | +574.2% | 7.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 9814 | 346.9 | 847.0 | 62.5 | 249.2 | 613.7 | -91.4% | 36.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c4_q1_p50_us | 161.5 | 19.4 | 1985 | 113.2 | 81.2 | 167.6 | +1129.1% | 22.5 | **N better** |
| N vs P+ | c4_q1_p99_us | 7194 | 1360 | 12436 | 2278 | 1876 | 17890 | +72.9% | 2.79 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 15233 | 2354 | 1697 | 72.9 | 1666 | 1780 | -88.9% | 8.13 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 45398 | 6026 | 10519 | 5514 | 5776 | 22393 | -76.8% | 6.04 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 550.9 | 41.9 | 1133 | 36.0 | 39.1 | 187.4 | +105.6% | 14.9 | **N better** |
| N vs P+ | write_p99_us | 2033 | 781.6 | 3213 | 267.0 | 584.1 | 2016 | +58.0% | 2.02 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1512 | 127.8 | 796.7 | 42.3 | 95.1 | 50.7 | -47.3% | 7.52 | **N better** |
| N vs P+ | pss_mib | 28.0 | 0.13 | 43.3 | 0.01 | 0.09 | 0.02 | +54.9% | 170.6 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 19881 | 863.1 | -2154 | 153.8 | 619.9 | 9352 | -110.8% | 35.5 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 168.8 | 15.3 | 143.8 | 15.8 | 15.5 | 36.4 | -14.8% | 1.61 | BELOW FLOOR |
| N vs P | q1_p99_us | 493.2 | 62.9 | 554.6 | 99.5 | 83.3 | 689.1 | +12.4% | 0.74 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 9386 | 507.2 | 250.1 | 24.1 | 359.0 | 1214 | -97.3% | 25.4 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2079 | 304.1 | 183.4 | 23.9 | 215.7 | 248.2 | -91.2% | 8.79 | **P better** |
| N vs P | q6_p50_us | 4018 | 264.9 | 842.1 | 171.8 | 223.3 | 775.3 | -79.0% | 14.2 | **P better** |
| N vs P | c2_q1_p50_us | 109.5 | 24.2 | 135.2 | 23.5 | 23.9 | 57.7 | +23.5% | 1.08 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 865.8 | 250.7 | 2256 | 425.3 | 349.0 | 716.7 | +160.6% | 3.98 | **N better** |
| N vs P | c2_q2_p50_us | 9814 | 346.9 | 240.5 | 41.8 | 247.1 | 540.1 | -97.5% | 38.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c4_q1_p50_us | 161.5 | 19.4 | 244.3 | 31.6 | 26.2 | 45.5 | +51.3% | 3.16 | **N better** |
| N vs P | c4_q1_p99_us | 7194 | 1360 | 2783 | 464.2 | 1016 | 3097 | -61.3% | 4.34 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 15233 | 2354 | 349.4 | 51.3 | 1665 | 1781 | -97.7% | 8.94 | **P better** |
| N vs P | c4_q2_p99_us | 45398 | 6026 | 3756 | 614.7 | 4283 | 1408 | -91.7% | 9.72 | **P better** |
| N vs P | write_p50_us | 550.9 | 41.9 | 20147 | 977.4 | 691.7 | 872.6 | +3557.1% | 28.3 | **N better** |
| N vs P | write_p99_us | 2033 | 781.6 | 34903 | 4103 | 2953 | 2049 | +1616.6% | 11.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1512 | 127.8 | 44.7 | 0.75 | 90.3 | 50.4 | -97.0% | 16.2 | **N better** |
| N vs P | pss_mib | 28.0 | 0.13 | 43.4 | 0.04 | 0.09 | 0.05 | +55.3% | 164.9 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 19881 | 863.1 | -2045 | 187.7 | 624.6 | 9325 | -110.3% | 35.1 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 689.8 | 32.3 | 143.8 | 15.8 | 25.4 | 14.3 | -79.2% | 21.5 | **P better** |
| P+ vs P | q1_p99_us | 2118 | 529.5 | 554.6 | 99.5 | 381.0 | 1879 | -73.8% | 4.10 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 536.9 | 25.4 | 250.1 | 24.1 | 24.7 | 107.0 | -53.4% | 11.6 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 358.2 | 52.4 | 271.3 | 25.5 | 41.2 | 72.1 | -24.3% | 2.11 | no difference |
| P+ vs P | q4_p50_us | 1967 | 141.8 | 314.5 | 17.2 | 101.0 | 14.2 | -84.0% | 16.4 | **P better** |
| P+ vs P | q5_p50_us | 3828 | 775.4 | 183.4 | 23.9 | 548.6 | 187.0 | -95.2% | 6.64 | **P better** |
| P+ vs P | q6_p50_us | 5095 | 429.8 | 842.1 | 171.8 | 327.3 | 445.8 | -83.5% | 13.0 | **P better** |
| P+ vs P | c2_q1_p50_us | 1080 | 88.9 | 135.2 | 23.5 | 65.0 | 60.2 | -87.5% | 14.5 | **P better** |
| P+ vs P | c2_q1_p99_us | 5837 | 932.6 | 2256 | 425.3 | 724.8 | 2339 | -61.3% | 4.94 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 847.0 | 62.5 | 240.5 | 41.8 | 53.2 | 353.8 | -71.6% | 11.4 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c4_q1_p50_us | 1985 | 113.2 | 244.3 | 31.6 | 83.1 | 167.5 | -87.7% | 21.0 | **P better** |
| P+ vs P | c4_q1_p99_us | 12436 | 2278 | 2783 | 464.2 | 1644 | 17620 | -77.6% | 5.87 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1697 | 72.9 | 349.4 | 51.3 | 63.0 | 59.8 | -79.4% | 21.4 | **P better** |
| P+ vs P | c4_q2_p99_us | 10519 | 5514 | 3756 | 614.7 | 3923 | 22350 | -64.3% | 1.72 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 1133 | 36.0 | 20147 | 977.4 | 691.6 | 853.3 | +1678.8% | 27.5 | **P+ better** |
| P+ vs P | write_p99_us | 3213 | 267.0 | 34903 | 4103 | 2907 | 995.2 | +986.3% | 10.9 | **P+ better** |
| P+ vs P | commits_per_s | 796.7 | 42.3 | 44.7 | 0.75 | 29.9 | 7.02 | -94.4% | 25.2 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.01 | 43.4 | 0.04 | 0.03 | 0.05 | +0.3% | 4.01 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2154 | 153.8 | -2045 | 187.7 | 171.6 | 2543 | -5.1% | 0.63 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 143.8 | 15.8 | 148.5 | 7.81 | 12.5 | 15.7 | +3.3% | 0.38 | BELOW FLOOR |
| P vs M | q1_p99_us | 554.6 | 99.5 | 547.0 | 86.1 | 93.1 | 689.9 | -1.4% | 0.08 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 250.1 | 24.1 | 228.9 | 18.6 | 21.5 | 28.3 | -8.5% | 0.99 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 271.3 | 25.5 | 287.0 | 42.8 | 35.2 | 95.2 | +5.8% | 0.45 | BELOW FLOOR |
| P vs M | q4_p50_us | 314.5 | 17.2 | 323.5 | 20.7 | 19.0 | 119.8 | +2.9% | 0.48 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q5_p50_us | 183.4 | 23.9 | 177.8 | 17.7 | 21.0 | 83.9 | -3.1% | 0.27 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | 842.1 | 171.8 | 879.2 | 72.0 | 131.7 | 121.4 | +4.4% | 0.28 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 135.2 | 23.5 | 160.8 | 21.7 | 22.7 | 74.7 | +18.9% | 1.13 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 2256 | 425.3 | 1589 | 398.9 | 412.3 | 207.6 | -29.6% | 1.62 | no difference |
| P vs M | c2_q2_p50_us | 240.5 | 41.8 | 248.8 | 35.8 | 38.9 | 141.9 | +3.5% | 0.21 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c4_q1_p50_us | 244.3 | 31.6 | 190.8 | 45.5 | 39.2 | 169.6 | -21.9% | 1.37 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2783 | 464.2 | 2364 | 105.4 | 336.6 | 138.0 | -15.0% | 1.24 | no difference |
| P vs M | c4_q2_p50_us | 349.4 | 51.3 | 315.5 | 37.3 | 44.9 | 132.1 | -9.7% | 0.76 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3756 | 614.7 | 3402 | 646.7 | 630.9 | 1269 | -9.4% | 0.56 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 20147 | 977.4 | 20226 | 962.1 | 969.8 | 1115 | +0.4% | 0.08 | BELOW FLOOR |
| P vs M | write_p99_us | 34903 | 4103 | 31443 | 3710 | 3912 | 9044 | -9.9% | 0.88 | BELOW FLOOR |
| P vs M | commits_per_s | 44.7 | 0.75 | 45.9 | 1.66 | 1.29 | 5.78 | +2.7% | 0.95 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.04 | 43.4 | 0.01 | 0.03 | 0.33 | -0.1% | 1.42 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2045 | 187.7 | -2090 | 154.1 | 171.7 | 6684 | +2.2% | 0.26 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 168.8 | 15.3 | 325.5 | 40.0 | 30.3 | 69.0 | +92.9% | 5.18 | **N better** |
| N vs H3 | q1_p99_us | 493.2 | 62.9 | 1012 | 237.3 | 173.6 | 1019 | +105.2% | 2.99 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 9386 | 507.2 | 354.7 | 33.7 | 359.4 | 1214 | -96.2% | 25.1 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2079 | 304.1 | 3935 | 566.3 | 454.5 | 660.3 | +89.3% | 4.08 | **N better** |
| N vs H3 | q6_p50_us | 4018 | 264.9 | 6138 | 682.4 | 517.6 | 1397 | +52.7% | 4.10 | **N better** |
| N vs H3 | c2_q1_p50_us | 109.5 | 24.2 | 361.8 | 31.0 | 27.8 | 156.1 | +230.5% | 9.07 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 865.8 | 250.7 | 1598 | 311.7 | 282.8 | 707.2 | +84.6% | 2.59 | no difference |
| N vs H3 | c2_q2_p50_us | 9814 | 346.9 | 366.7 | 23.4 | 245.8 | 530.5 | -96.3% | 38.4 | **H3 better** |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c4_q1_p50_us | 161.5 | 19.4 | 529.9 | 98.0 | 70.6 | 165.9 | +228.1% | 5.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 7194 | 1360 | 1889 | 273.2 | 980.8 | 3096 | -73.7% | 5.41 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 15233 | 2354 | 573.1 | 77.2 | 1666 | 1839 | -96.2% | 8.80 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 45398 | 6026 | 2100 | 397.9 | 4270 | 1592 | -95.4% | 10.1 | **H3 better** |
| N vs H3 | write_p50_us | 550.9 | 41.9 | 1580 | 108.2 | 82.1 | 286.8 | +186.8% | 12.5 | **N better** |
| N vs H3 | write_p99_us | 2033 | 781.6 | 6135 | 2084 | 1574 | 2959 | +201.7% | 2.61 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1512 | 127.8 | 476.3 | 77.2 | 105.5 | 83.6 | -68.5% | 9.81 | **N better** |
| N vs H3 | pss_mib | 28.0 | 0.13 | 67.6 | 0.30 | 0.23 | 0.15 | +141.9% | 173.5 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 19881 | 863.1 | 13555 | 560.4 | 727.7 | 12699 | -31.8% | 8.69 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 689.8 | 32.3 | 325.5 | 40.0 | 36.4 | 60.4 | -52.8% | 10.0 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2118 | 529.5 | 1012 | 237.3 | 410.3 | 2023 | -52.2% | 2.69 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 536.9 | 25.4 | 354.7 | 33.7 | 29.8 | 112.1 | -33.9% | 6.10 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 358.2 | 52.4 | 438.6 | 50.8 | 51.6 | 35.5 | +22.4% | 1.56 | no difference |
| P+ vs H3 | q4_p50_us | 1967 | 141.8 | 1944 | 269.7 | 215.4 | 254.9 | -1.1% | 0.10 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3828 | 775.4 | 3935 | 566.3 | 679.0 | 639.8 | +2.8% | 0.16 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 5095 | 429.8 | 6138 | 682.4 | 570.2 | 1245 | +20.5% | 1.83 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 1080 | 88.9 | 361.8 | 31.0 | 66.6 | 157.1 | -66.5% | 10.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 5837 | 932.6 | 1598 | 311.7 | 695.3 | 2336 | -72.6% | 6.10 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p50_us | 847.0 | 62.5 | 366.7 | 23.4 | 47.2 | 338.8 | -56.7% | 10.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c4_q1_p50_us | 1985 | 113.2 | 529.9 | 98.0 | 105.8 | 231.4 | -73.3% | 13.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 12436 | 2278 | 1889 | 273.2 | 1623 | 17620 | -84.8% | 6.50 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1697 | 72.9 | 573.1 | 77.2 | 75.0 | 462.8 | -66.2% | 15.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 10519 | 5514 | 2100 | 397.9 | 3909 | 22362 | -80.0% | 2.15 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1133 | 36.0 | 1580 | 108.2 | 80.7 | 221.2 | +39.5% | 5.55 | **P+ better** |
| P+ vs H3 | write_p99_us | 3213 | 267.0 | 6135 | 2084 | 1485 | 2355 | +91.0% | 1.97 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 796.7 | 42.3 | 476.3 | 77.2 | 62.2 | 67.1 | -40.2% | 5.15 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.01 | 67.6 | 0.30 | 0.21 | 0.15 | +56.2% | 115.6 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2154 | 153.8 | 13555 | 560.4 | 410.9 | 8986 | -729.3% | 38.2 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 168.8 | 15.3 | 1378 | 67.2 | 48.7 | 142.5 | +716.4% | 24.8 | **N better** |
| N vs T | q1_p99_us | 493.2 | 62.9 | 3748 | 1031 | 730.5 | 545.8 | +659.8% | 4.46 | **N better** |
| N vs T | q2_p50_us | 9386 | 507.2 | 1396 | 69.4 | 362.0 | 1246 | -85.1% | 22.1 | **T better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 109.5 | 24.2 | 1873 | 121.7 | 87.7 | 189.5 | +1610.8% | 20.1 | **N better** |
| N vs T | c2_q1_p99_us | 865.8 | 250.7 | 4846 | 665.1 | 502.6 | 707.8 | +459.7% | 7.92 | **N better** |
| N vs T | c2_q2_p50_us | 9814 | 346.9 | 1876 | 111.2 | 257.6 | 534.0 | -80.9% | 30.8 | **T better** |
| N vs T | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c4_q1_p50_us | 161.5 | 19.4 | 2234 | 125.5 | 89.8 | 153.0 | +1283.1% | 23.1 | **N better** |
| N vs T | c4_q1_p99_us | 7194 | 1360 | 7057 | 2546 | 2041 | 3995 | -1.9% | 0.07 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c4_q2_p50_us | 15233 | 2354 | 2243 | 146.0 | 1668 | 1780 | -85.3% | 7.79 | **T better** |
| N vs T | c4_q2_p99_us | 45398 | 6026 | 6933 | 2606 | 4642 | 1539 | -84.7% | 8.29 | **T better** |
| N vs T | write_p50_us | 550.9 | 41.9 | 1336 | 77.7 | 62.4 | 237.3 | +142.6% | 12.6 | **N better** |
| N vs T | write_p99_us | 2033 | 781.6 | 2222 | 321.7 | 597.7 | 2000 | +9.3% | 0.32 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | commits_per_s | 1512 | 127.8 | 708.0 | 77.9 | 105.8 | 201.9 | -53.2% | 7.60 | **N better** |
| N vs T | pss_mib | 28.0 | 0.13 | 2524 | 0.03 | 0.09 | 0.13 | +8928.7% | 26997 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 19881 | 863.1 | -2069 | 167.2 | 621.7 | 9412 | -110.4% | 35.3 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 689.8 | 32.3 | 1378 | 67.2 | 52.7 | 138.6 | +99.8% | 13.1 | **P+ better** |
| P+ vs T | q1_p99_us | 2118 | 529.5 | 3748 | 1031 | 819.6 | 1831 | +77.0% | 1.99 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | q2_p50_us | 536.9 | 25.4 | 1396 | 69.4 | 52.3 | 299.3 | +160.0% | 16.4 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 1080 | 88.9 | 1873 | 121.7 | 106.6 | 190.3 | +73.5% | 7.44 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 5837 | 932.6 | 4846 | 665.1 | 810.0 | 2336 | -17.0% | 1.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c2_q2_p50_us | 847.0 | 62.5 | 1876 | 111.2 | 90.2 | 344.4 | +121.5% | 11.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c4_q1_p50_us | 1985 | 113.2 | 2234 | 125.5 | 119.5 | 222.3 | +12.5% | 2.08 | no difference |
| P+ vs T | c4_q1_p99_us | 12436 | 2278 | 7057 | 2546 | 2416 | 17800 | -43.3% | 2.23 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c4_q2_p50_us | 1697 | 72.9 | 2243 | 146.0 | 115.4 | 11.4 | +32.2% | 4.73 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 10519 | 5514 | 6933 | 2606 | 4312 | 22358 | -34.1% | 0.83 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | write_p50_us | 1133 | 36.0 | 1336 | 77.7 | 60.5 | 151.6 | +18.0% | 3.37 | **P+ better** |
| P+ vs T | write_p99_us | 3213 | 267.0 | 2222 | 321.7 | 295.6 | 890.2 | -30.8% | 3.35 | **T better** |
| P+ vs T | commits_per_s | 796.7 | 42.3 | 708.0 | 77.9 | 62.7 | 195.7 | -11.1% | 1.41 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.01 | 2524 | 0.03 | 0.02 | 0.13 | +5729.2% | 108904 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -2154 | 153.8 | -2069 | 167.2 | 160.6 | 2845 | -3.9% | 0.53 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 325.5 | 40.0 | 1378 | 67.2 | 55.3 | 150.5 | +323.3% | 19.0 | **H3 better** |
| H3 vs T | q1_p99_us | 1012 | 237.3 | 3748 | 1031 | 748.2 | 1155 | +270.3% | 3.66 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q2_p50_us | 354.7 | 33.7 | 1396 | 69.4 | 54.6 | 282.8 | +293.5% | 19.1 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 361.8 | 31.0 | 1873 | 121.7 | 88.8 | 238.3 | +417.7% | 17.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q1_p99_us | 1598 | 311.7 | 4846 | 665.1 | 519.4 | 30.0 | +203.2% | 6.25 | **H3 better** |
| H3 vs T | c2_q2_p50_us | 366.7 | 23.4 | 1876 | 111.2 | 80.3 | 152.7 | +411.6% | 18.8 | **H3 better** |
| H3 vs T | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c4_q1_p50_us | 529.9 | 98.0 | 2234 | 125.5 | 112.6 | 221.0 | +321.5% | 15.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 1889 | 273.2 | 7057 | 2546 | 1810 | 2524 | +273.7% | 2.86 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c4_q2_p50_us | 573.1 | 77.2 | 2243 | 146.0 | 116.7 | 463.0 | +291.5% | 14.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p99_us | 2100 | 397.9 | 6933 | 2606 | 1864 | 980.7 | +230.1% | 2.59 | no difference |
| H3 vs T | write_p50_us | 1580 | 108.2 | 1336 | 77.7 | 94.2 | 264.8 | -15.4% | 2.58 | BELOW FLOOR |
| H3 vs T | write_p99_us | 6135 | 2084 | 2222 | 321.7 | 1491 | 2342 | -63.8% | 2.63 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | commits_per_s | 476.3 | 77.2 | 708.0 | 77.9 | 77.5 | 206.6 | +48.7% | 2.99 | no difference |
| H3 vs T | pss_mib | 67.6 | 0.30 | 2524 | 0.03 | 0.21 | 0.20 | +3632.5% | 11617 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 13555 | 560.4 | -2069 | 167.2 | 413.5 | 9049 | -115.3% | 37.8 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 168.8 | 15.3 | 901.3 | 54.5 | 40.0 | 291.5 | +434.0% | 18.3 | **N better** |
| N vs H1 | q1_p99_us | 493.2 | 62.9 | 3408 | 497.4 | 354.5 | 1212 | +591.0% | 8.22 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q2_p50_us | 9386 | 507.2 | 825.1 | 86.7 | 363.8 | 1231 | -91.2% | 23.5 | **H1 better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2079 | 304.1 | 4348 | 1034 | 762.3 | 3392 | +109.2% | 2.98 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q6_p50_us | 4018 | 264.9 | 1761 | 238.9 | 252.2 | 829.5 | -56.2% | 8.95 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 109.5 | 24.2 | 810.4 | 62.8 | 47.6 | 46.7 | +640.3% | 14.7 | **N better** |
| N vs H1 | c2_q1_p99_us | 865.8 | 250.7 | 2788 | 611.9 | 467.6 | 1067 | +222.0% | 4.11 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c2_q2_p50_us | 9814 | 346.9 | 840.0 | 78.4 | 251.5 | 540.8 | -91.4% | 35.7 | **H1 better** |
| N vs H1 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | c4_q1_p50_us | 161.5 | 19.4 | 881.8 | 90.9 | 65.7 | 79.6 | +446.0% | 11.0 | **N better** |
| N vs H1 | c4_q1_p99_us | 7194 | 1360 | 2851 | 395.3 | 1001 | 3107 | -60.4% | 4.34 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p50_us | 15233 | 2354 | 902.2 | 44.3 | 1665 | 1780 | -94.1% | 8.61 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 45398 | 6026 | 3224 | 813.9 | 4299 | 1408 | -92.9% | 9.81 | **H1 better** |
| N vs H1 | write_p50_us | 550.9 | 41.9 | 1690 | 90.9 | 70.8 | 200.7 | +206.8% | 16.1 | **N better** |
| N vs H1 | write_p99_us | 2033 | 781.6 | 4777 | 655.1 | 721.1 | 1935 | +135.0% | 3.81 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | commits_per_s | 1512 | 127.8 | 528.0 | 32.1 | 93.1 | 50.3 | -65.1% | 10.6 | **N better** |
| N vs H1 | pss_mib | 28.0 | 0.13 | 288.3 | 5.66 | 4.00 | 3.26 | +931.2% | 65.0 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 19881 | 863.1 | 33773 | 1843 | 1439 | 15203 | +69.9% | 9.65 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| P+ vs H1 | q1_p50_us | 689.8 | 32.3 | 901.3 | 54.5 | 44.8 | 289.5 | +30.7% | 4.72 | BELOW FLOOR |
| P+ vs H1 | q1_p99_us | 2118 | 529.5 | 3408 | 497.4 | 513.7 | 2127 | +61.0% | 2.51 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | q2_p50_us | 536.9 | 25.4 | 825.1 | 86.7 | 63.9 | 232.1 | +53.7% | 4.51 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 358.2 | 52.4 | 866.9 | 42.5 | 47.7 | 122.5 | +142.0% | 10.7 | **P+ better** |
| P+ vs H1 | q4_p50_us | 1967 | 141.8 | 3296 | 289.1 | 227.7 | 111.6 | +67.6% | 5.84 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3828 | 775.4 | 4348 | 1034 | 914.0 | 3388 | +13.6% | 0.57 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q6_p50_us | 5095 | 429.8 | 1761 | 238.9 | 347.7 | 534.6 | -65.4% | 9.59 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 1080 | 88.9 | 810.4 | 62.8 | 76.9 | 49.8 | -24.9% | 3.50 | **H1 better** |
| P+ vs H1 | c2_q1_p99_us | 5837 | 932.6 | 2788 | 611.9 | 788.7 | 2469 | -52.2% | 3.87 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c2_q2_p50_us | 847.0 | 62.5 | 840.0 | 78.4 | 70.9 | 354.8 | -0.8% | 0.10 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | c4_q1_p50_us | 1985 | 113.2 | 881.8 | 90.9 | 102.6 | 179.8 | -55.6% | 10.8 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 12436 | 2278 | 2851 | 395.3 | 1635 | 17622 | -77.1% | 5.86 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q2_p50_us | 1697 | 72.9 | 902.2 | 44.3 | 60.3 | 47.9 | -46.8% | 13.2 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 10519 | 5514 | 3224 | 813.9 | 3941 | 22350 | -69.3% | 1.85 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | write_p50_us | 1133 | 36.0 | 1690 | 90.9 | 69.1 | 83.4 | +49.2% | 8.07 | **P+ better** |
| P+ vs H1 | write_p99_us | 3213 | 267.0 | 4777 | 655.1 | 500.2 | 733.9 | +48.7% | 3.13 | **P+ better** |
| P+ vs H1 | commits_per_s | 796.7 | 42.3 | 528.0 | 32.1 | 37.5 | 6.60 | -33.7% | 7.16 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.01 | 288.3 | 5.66 | 4.00 | 3.26 | +565.8% | 61.2 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -2154 | 153.8 | 33773 | 1843 | 1307 | 12273 | -1667.9% | 27.5 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 168.8 | 15.3 | 142.9 | 7.20 | 12.0 | 50.3 | -15.3% | 2.16 | BELOW FLOOR |
| N vs H2 | q1_p99_us | 493.2 | 62.9 | 563.1 | 203.2 | 150.4 | 37.8 | +14.2% | 0.46 | no difference |
| N vs H2 | q2_p50_us | 9386 | 507.2 | 232.6 | 13.5 | 358.8 | 1214 | -97.5% | 25.5 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2079 | 304.1 | 175.8 | 16.1 | 215.4 | 248.7 | -91.5% | 8.84 | **H2 better** |
| N vs H2 | q6_p50_us | 4018 | 264.9 | 884.2 | 129.0 | 208.4 | 789.4 | -78.0% | 15.0 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 109.5 | 24.2 | 176.5 | 49.8 | 39.1 | 48.0 | +61.2% | 1.71 | no difference |
| N vs H2 | c2_q1_p99_us | 865.8 | 250.7 | 1700 | 656.3 | 496.8 | 2374 | +96.4% | 1.68 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q2_p50_us | 9814 | 346.9 | 265.8 | 72.5 | 250.6 | 524.8 | -97.3% | 38.1 | **H2 better** |
| N vs H2 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c4_q1_p50_us | 161.5 | 19.4 | 250.3 | 52.4 | 39.5 | 255.7 | +55.0% | 2.25 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p99_us | 7194 | 1360 | 2745 | 311.7 | 986.5 | 3413 | -61.8% | 4.51 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q2_p50_us | 15233 | 2354 | 327.4 | 19.5 | 1665 | 1780 | -97.9% | 8.95 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 45398 | 6026 | 2918 | 464.1 | 4273 | 1851 | -93.6% | 9.94 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 550.9 | 41.9 | 1617 | 128.7 | 95.7 | 201.3 | +193.6% | 11.1 | **N better** |
| N vs H2 | write_p99_us | 2033 | 781.6 | 6575 | 622.4 | 706.5 | 1907 | +223.4% | 6.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1512 | 127.8 | 520.2 | 43.4 | 95.4 | 98.1 | -65.6% | 10.4 | **N better** |
| N vs H2 | pss_mib | 28.0 | 0.13 | 43.4 | 0.01 | 0.09 | 0.00 | +55.2% | 171.7 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 19881 | 863.1 | -2210 | 195.2 | 625.7 | 9351 | -111.1% | 35.3 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 689.8 | 32.3 | 142.9 | 7.20 | 23.4 | 37.5 | -79.3% | 23.4 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2118 | 529.5 | 563.1 | 203.2 | 401.0 | 1748 | -73.4% | 3.88 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 536.9 | 25.4 | 232.6 | 13.5 | 20.3 | 107.4 | -56.7% | 15.0 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 358.2 | 52.4 | 255.7 | 39.4 | 46.4 | 185.0 | -28.6% | 2.21 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q4_p50_us | 1967 | 141.8 | 302.9 | 24.7 | 101.8 | 167.4 | -84.6% | 16.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q5_p50_us | 3828 | 775.4 | 175.8 | 16.1 | 548.4 | 187.6 | -95.4% | 6.66 | **H2 better** |
| P+ vs H2 | q6_p50_us | 5095 | 429.8 | 884.2 | 129.0 | 317.3 | 469.9 | -82.6% | 13.3 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 1080 | 88.9 | 176.5 | 49.8 | 72.0 | 51.0 | -83.7% | 12.5 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5837 | 932.6 | 1700 | 656.3 | 806.4 | 3254 | -70.9% | 5.13 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 847.0 | 62.5 | 265.8 | 72.5 | 67.7 | 329.9 | -68.6% | 8.59 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c4_q1_p50_us | 1985 | 113.2 | 250.3 | 52.4 | 88.2 | 302.3 | -87.4% | 19.7 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 12436 | 2278 | 2745 | 311.7 | 1626 | 17679 | -77.9% | 5.96 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c4_q2_p50_us | 1697 | 72.9 | 327.4 | 19.5 | 53.3 | 19.0 | -80.7% | 25.7 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 10519 | 5514 | 2918 | 464.1 | 3913 | 22382 | -72.3% | 1.94 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1133 | 36.0 | 1617 | 128.7 | 94.5 | 84.9 | +42.8% | 5.13 | **P+ better** |
| P+ vs H2 | write_p99_us | 3213 | 267.0 | 6575 | 622.4 | 478.9 | 657.2 | +104.6% | 7.02 | **P+ better** |
| P+ vs H2 | commits_per_s | 796.7 | 42.3 | 520.2 | 43.4 | 42.8 | 84.4 | -34.7% | 6.46 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.01 | 43.4 | 0.01 | 0.01 | 0.02 | +0.2% | 12.4 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -2154 | 153.8 | -2210 | 195.2 | 175.7 | 2634 | +2.6% | 0.32 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 170.8 | 9.80 | 723.5 | 46.5 | 33.6 | 164.5 | +323.5% | 16.5 | **N better** |
| N vs P+ | q1_p99_us | 490.9 | 58.0 | 2077 | 221.0 | 161.6 | 234.0 | +323.1% | 9.81 | **N better** |
| N vs P+ | q2_p50_us | 9817 | 242.5 | 580.8 | 74.1 | 179.3 | 147.4 | -94.1% | 51.5 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2337 | 74.0 | 3798 | 320.4 | 232.5 | 125.1 | +62.5% | 6.28 | **N better** |
| N vs P+ | q6_p50_us | 4367 | 358.1 | 4883 | 690.1 | 549.7 | 414.0 | +11.8% | 0.94 | no difference |
| N vs P+ | c2_q1_p50_us | 140.6 | 15.1 | 972.7 | 80.0 | 57.6 | 37.8 | +592.0% | 14.5 | **N better** |
| N vs P+ | c2_q1_p99_us | 2454 | 918.0 | 6578 | 1147 | 1039 | 2471 | +168.1% | 3.97 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 10040 | 347.4 | 781.1 | 45.2 | 247.7 | 2313 | -92.2% | 37.4 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 156.1 | 25.2 | 2110 | 133.5 | 96.1 | 113.9 | +1251.7% | 20.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 6951 | 1278 | 12276 | 3227 | 2454 | 7671 | +76.6% | 2.17 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 14017 | 3418 | 1925 | 122.8 | 2418 | 4964 | -86.3% | 5.00 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 39671 | 3363 | 11470 | 2528 | 2975 | 6338 | -71.1% | 9.48 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 600.8 | 46.5 | 1144 | 71.6 | 60.3 | 122.6 | +90.4% | 9.00 | **N better** |
| N vs P+ | write_p99_us | 1325 | 509.5 | 3210 | 262.6 | 405.3 | 487.5 | +142.3% | 4.65 | **N better** |
| N vs P+ | commits_per_s | 1500 | 124.8 | 750.4 | 49.4 | 94.9 | 536.3 | -50.0% | 7.90 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 25.0 | 2.53 | 43.4 | 0.01 | 1.79 | 1.72 | +73.8% | 10.3 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 11802 | 3180 | -2032 | 133.7 | 2250 | 4401 | -117.2% | 6.15 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 170.8 | 9.80 | 154.6 | 5.21 | 7.85 | 6.73 | -9.5% | 2.07 | no difference |
| N vs P | q1_p99_us | 490.9 | 58.0 | 792.2 | 337.2 | 241.9 | 238.0 | +61.4% | 1.25 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 9817 | 242.5 | 254.3 | 18.0 | 172.0 | 148.8 | -97.4% | 55.6 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2337 | 74.0 | 180.8 | 16.9 | 53.7 | 175.9 | -92.3% | 40.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 4367 | 358.1 | 860.5 | 165.2 | 278.9 | 277.8 | -80.3% | 12.6 | **P better** |
| N vs P | c2_q1_p50_us | 140.6 | 15.1 | 133.4 | 26.7 | 21.7 | 62.5 | -5.1% | 0.33 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 2454 | 918.0 | 2279 | 516.9 | 744.9 | 1872 | -7.1% | 0.23 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 10040 | 347.4 | 215.6 | 47.5 | 248.0 | 2315 | -97.9% | 39.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 156.1 | 25.2 | 249.8 | 40.4 | 33.6 | 45.6 | +60.0% | 2.78 | no difference |
| N vs P | c4_q1_p99_us | 6951 | 1278 | 2893 | 447.9 | 957.8 | 2499 | -58.4% | 4.24 | **P better** |
| N vs P | c4_q2_p50_us | 14017 | 3418 | 376.4 | 55.0 | 2417 | 4960 | -97.3% | 5.64 | **P better** |
| N vs P | c4_q2_p99_us | 39671 | 3363 | 3536 | 941.3 | 2469 | 5767 | -91.1% | 14.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 600.8 | 46.5 | 20368 | 539.6 | 382.9 | 362.1 | +3290.1% | 51.6 | **N better** |
| N vs P | write_p99_us | 1325 | 509.5 | 32612 | 3704 | 2644 | 770.1 | +2361.8% | 11.8 | **N better** |
| N vs P | commits_per_s | 1500 | 124.8 | 47.0 | 1.72 | 88.3 | 533.1 | -96.9% | 16.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_mib | 25.0 | 2.53 | 43.3 | 0.02 | 1.79 | 1.72 | +73.5% | 10.3 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 11802 | 3180 | -2047 | 143.7 | 2251 | 4420 | -117.3% | 6.15 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 723.5 | 46.5 | 154.6 | 5.21 | 33.1 | 164.6 | -78.6% | 17.2 | **P better** |
| P+ vs P | q1_p99_us | 2077 | 221.0 | 792.2 | 337.2 | 285.1 | 293.0 | -61.9% | 4.51 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 580.8 | 74.1 | 254.3 | 18.0 | 53.9 | 37.5 | -56.2% | 6.05 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 383.0 | 18.9 | 286.1 | 28.7 | 24.3 | 95.8 | -25.3% | 3.99 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q4_p50_us | 1766 | 111.6 | 301.7 | 50.0 | 86.5 | 313.4 | -82.9% | 16.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q5_p50_us | 3798 | 320.4 | 180.8 | 16.9 | 226.8 | 132.8 | -95.2% | 15.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 4883 | 690.1 | 860.5 | 165.2 | 501.7 | 382.6 | -82.4% | 8.02 | **P better** |
| P+ vs P | c2_q1_p50_us | 972.7 | 80.0 | 133.4 | 26.7 | 59.6 | 71.0 | -86.3% | 14.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 6578 | 1147 | 2279 | 516.9 | 889.4 | 2044 | -65.4% | 4.83 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 781.1 | 45.2 | 215.6 | 47.5 | 46.4 | 97.0 | -72.4% | 12.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 2110 | 133.5 | 249.8 | 40.4 | 98.6 | 111.5 | -88.2% | 18.9 | **P better** |
| P+ vs P | c4_q1_p99_us | 12276 | 3227 | 2893 | 447.9 | 2303 | 7292 | -76.4% | 4.07 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1925 | 122.8 | 376.4 | 55.0 | 95.1 | 184.5 | -80.4% | 16.3 | **P better** |
| P+ vs P | c4_q2_p99_us | 11470 | 2528 | 3536 | 941.3 | 1907 | 4445 | -69.2% | 4.16 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 1144 | 71.6 | 20368 | 539.6 | 384.9 | 376.9 | +1680.9% | 50.0 | **P+ better** |
| P+ vs P | write_p99_us | 3210 | 262.6 | 32612 | 3704 | 2626 | 884.3 | +915.8% | 11.2 | **P+ better** |
| P+ vs P | commits_per_s | 750.4 | 49.4 | 47.0 | 1.72 | 34.9 | 59.1 | -93.7% | 20.1 | **P+ better** |
| P+ vs P | pss_mib | 43.4 | 0.01 | 43.3 | 0.02 | 0.02 | 0.03 | -0.2% | 5.35 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2032 | 133.7 | -2047 | 143.7 | 138.8 | 2460 | +0.7% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 154.6 | 5.21 | 152.0 | 3.82 | 4.57 | 19.2 | -1.7% | 0.56 | BELOW FLOOR |
| P vs M | q1_p99_us | 792.2 | 337.2 | 700.1 | 246.0 | 295.2 | 942.5 | -11.6% | 0.31 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 254.3 | 18.0 | 252.3 | 19.1 | 18.5 | 58.0 | -0.8% | 0.11 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 286.1 | 28.7 | 294.1 | 20.4 | 24.9 | 150.7 | +2.8% | 0.32 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 301.7 | 50.0 | 335.9 | 22.1 | 38.7 | 135.6 | +11.3% | 0.88 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q5_p50_us | 180.8 | 16.9 | 180.5 | 22.6 | 19.9 | 131.3 | -0.2% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 860.5 | 165.2 | 792.2 | 46.7 | 121.4 | 167.8 | -7.9% | 0.56 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 133.4 | 26.7 | 138.2 | 35.6 | 31.4 | 61.4 | +3.6% | 0.15 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 2279 | 516.9 | 1690 | 430.3 | 475.6 | 893.5 | -25.9% | 1.24 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 215.6 | 47.5 | 219.0 | 33.3 | 41.0 | 106.4 | +1.6% | 0.08 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 249.8 | 40.4 | 238.9 | 34.2 | 37.4 | 30.1 | -4.3% | 0.29 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2893 | 447.9 | 2859 | 529.7 | 490.5 | 533.2 | -1.2% | 0.07 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 376.4 | 55.0 | 337.8 | 42.4 | 49.1 | 43.5 | -10.3% | 0.79 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3536 | 941.3 | 2942 | 293.8 | 697.3 | 2570 | -16.8% | 0.85 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 20368 | 539.6 | 20918 | 852.4 | 713.3 | 702.9 | +2.7% | 0.77 | BELOW FLOOR |
| P vs M | write_p99_us | 32612 | 3704 | 33318 | 1190 | 2751 | 2160 | +2.2% | 0.26 | BELOW FLOOR |
| P vs M | commits_per_s | 47.0 | 1.72 | 43.8 | 1.85 | 1.79 | 1.05 | -6.9% | 1.81 | no difference |
| P vs M | pss_mib | 43.3 | 0.02 | 43.5 | 0.01 | 0.01 | 0.15 | +0.3% | 8.23 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2047 | 143.7 | -1989 | 120.8 | 132.7 | 2644 | -2.9% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 170.8 | 9.80 | 319.0 | 23.2 | 17.8 | 30.2 | +86.7% | 8.31 | **N better** |
| N vs H3 | q1_p99_us | 490.9 | 58.0 | 934.6 | 262.9 | 190.4 | 547.9 | +90.4% | 2.33 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 9817 | 242.5 | 331.2 | 19.9 | 172.1 | 146.3 | -96.6% | 55.1 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2337 | 74.0 | 3981 | 193.9 | 146.8 | 1299 | +70.3% | 11.2 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q6_p50_us | 4367 | 358.1 | 5797 | 802.1 | 621.1 | 540.9 | +32.7% | 2.30 | no difference |
| N vs H3 | c2_q1_p50_us | 140.6 | 15.1 | 350.6 | 57.6 | 42.1 | 129.0 | +149.4% | 4.99 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 2454 | 918.0 | 1286 | 278.6 | 678.3 | 1650 | -47.6% | 1.72 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 10040 | 347.4 | 347.7 | 40.1 | 247.3 | 2314 | -96.5% | 39.2 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 156.1 | 25.2 | 579.8 | 40.9 | 33.9 | 36.2 | +271.4% | 12.5 | **N better** |
| N vs H3 | c4_q1_p99_us | 6951 | 1278 | 2038 | 334.2 | 934.3 | 6002 | -70.7% | 5.26 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 14017 | 3418 | 600.8 | 31.8 | 2417 | 4962 | -95.7% | 5.55 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 39671 | 3363 | 2049 | 377.8 | 2393 | 5626 | -94.8% | 15.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 600.8 | 46.5 | 1458 | 148.0 | 109.7 | 85.0 | +142.7% | 7.82 | **N better** |
| N vs H3 | write_p99_us | 1325 | 509.5 | 4457 | 646.0 | 581.8 | 165.3 | +236.4% | 5.38 | **N better** |
| N vs H3 | commits_per_s | 1500 | 124.8 | 535.3 | 120.9 | 122.9 | 533.2 | -64.3% | 7.85 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 25.0 | 2.53 | 67.7 | 0.04 | 1.79 | 1.73 | +170.9% | 23.9 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 11802 | 3180 | 13233 | 778.5 | 2315 | 11090 | +12.1% | 0.62 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 723.5 | 46.5 | 319.0 | 23.2 | 36.7 | 167.2 | -55.9% | 11.0 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2077 | 221.0 | 934.6 | 262.9 | 242.9 | 574.0 | -55.0% | 4.70 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 580.8 | 74.1 | 331.2 | 19.9 | 54.3 | 26.1 | -43.0% | 4.60 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 383.0 | 18.9 | 371.5 | 32.0 | 26.3 | 94.4 | -3.0% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | 1766 | 111.6 | 1764 | 82.1 | 97.9 | 302.0 | -0.1% | 0.02 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3798 | 320.4 | 3981 | 193.9 | 264.8 | 1293 | +4.8% | 0.69 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q6_p50_us | 4883 | 690.1 | 5797 | 802.1 | 748.2 | 601.5 | +18.7% | 1.22 | no difference |
| P+ vs H3 | c2_q1_p50_us | 972.7 | 80.0 | 350.6 | 57.6 | 69.7 | 133.4 | -64.0% | 8.93 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 6578 | 1147 | 1286 | 278.6 | 834.4 | 1843 | -80.5% | 6.34 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 781.1 | 45.2 | 347.7 | 40.1 | 42.7 | 73.2 | -55.5% | 10.2 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2110 | 133.5 | 579.8 | 40.9 | 98.7 | 108.0 | -72.5% | 15.5 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 12276 | 3227 | 2038 | 334.2 | 2294 | 9108 | -83.4% | 4.46 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1925 | 122.8 | 600.8 | 31.8 | 89.7 | 227.3 | -68.8% | 14.8 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 11470 | 2528 | 2049 | 377.8 | 1807 | 4261 | -82.1% | 5.21 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1144 | 71.6 | 1458 | 148.0 | 116.3 | 134.7 | +27.5% | 2.71 | no difference |
| P+ vs H3 | write_p99_us | 3210 | 262.6 | 4457 | 646.0 | 493.1 | 465.0 | +38.8% | 2.53 | no difference |
| P+ vs H3 | commits_per_s | 750.4 | 49.4 | 535.3 | 120.9 | 92.3 | 60.7 | -28.7% | 2.33 | no difference |
| P+ vs H3 | pss_mib | 43.4 | 0.01 | 67.7 | 0.04 | 0.03 | 0.22 | +55.9% | 809.2 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2032 | 133.7 | 13233 | 778.5 | 558.6 | 10464 | -751.1% | 27.3 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 170.8 | 9.80 | 1396 | 93.5 | 66.5 | 102.7 | +717.0% | 18.4 | **N better** |
| N vs T | q1_p99_us | 490.9 | 58.0 | 3706 | 891.8 | 631.9 | 396.7 | +654.9% | 5.09 | **N better** |
| N vs T | q2_p50_us | 9817 | 242.5 | 1436 | 79.5 | 180.5 | 149.5 | -85.4% | 46.4 | **T better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 140.6 | 15.1 | 1979 | 185.2 | 131.4 | 66.7 | +1307.6% | 14.0 | **N better** |
| N vs T | c2_q1_p99_us | 2454 | 918.0 | 4489 | 1105 | 1016 | 12949 | +82.9% | 2.00 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c2_q2_p50_us | 10040 | 347.4 | 1985 | 190.9 | 280.3 | 2314 | -80.2% | 28.7 | **T better** |
| N vs T | c4_q1_p50_us | 156.1 | 25.2 | 2367 | 191.1 | 136.3 | 144.7 | +1415.9% | 16.2 | **N better** |
| N vs T | c4_q1_p99_us | 6951 | 1278 | 6929 | 2569 | 2029 | 3033 | -0.3% | 0.01 | BELOW FLOOR |
| N vs T | c4_q2_p50_us | 14017 | 3418 | 2450 | 167.0 | 2420 | 4964 | -82.5% | 4.78 | **T better** |
| N vs T | c4_q2_p99_us | 39671 | 3363 | 6893 | 2237 | 2856 | 5428 | -82.6% | 11.5 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 600.8 | 46.5 | 1413 | 137.0 | 102.3 | 45.5 | +135.2% | 7.94 | **N better** |
| N vs T | write_p99_us | 1325 | 509.5 | 2505 | 354.7 | 439.0 | 187.0 | +89.1% | 2.69 | no difference |
| N vs T | commits_per_s | 1500 | 124.8 | 674.0 | 49.3 | 94.9 | 533.4 | -55.1% | 8.70 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | pss_mib | 25.0 | 2.53 | 2525 | 0.02 | 1.79 | 1.72 | +10008.6% | 1397 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 11802 | 3180 | -1748 | 126.3 | 2250 | 4433 | -114.8% | 6.02 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 723.5 | 46.5 | 1396 | 93.5 | 73.8 | 193.9 | +92.9% | 9.10 | **P+ better** |
| P+ vs T | q1_p99_us | 2077 | 221.0 | 3706 | 891.8 | 649.7 | 432.0 | +78.4% | 2.51 | no difference |
| P+ vs T | q2_p50_us | 580.8 | 74.1 | 1436 | 79.5 | 76.9 | 40.4 | +147.2% | 11.1 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 972.7 | 80.0 | 1979 | 185.2 | 142.6 | 74.7 | +103.4% | 7.05 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 6578 | 1147 | 4489 | 1105 | 1126 | 12975 | -31.8% | 1.86 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c2_q2_p50_us | 781.1 | 45.2 | 1985 | 190.9 | 138.7 | 61.6 | +154.2% | 8.68 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2110 | 133.5 | 2367 | 191.1 | 164.8 | 176.9 | +12.1% | 1.55 | no difference |
| P+ vs T | c4_q1_p99_us | 12276 | 3227 | 6929 | 2569 | 2916 | 7492 | -43.6% | 1.83 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q2_p50_us | 1925 | 122.8 | 2450 | 167.0 | 146.6 | 265.8 | +27.3% | 3.58 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 11470 | 2528 | 6893 | 2237 | 2387 | 3995 | -39.9% | 1.92 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | write_p50_us | 1144 | 71.6 | 1413 | 137.0 | 109.3 | 114.0 | +23.6% | 2.47 | no difference |
| P+ vs T | write_p99_us | 3210 | 262.6 | 2505 | 354.7 | 312.1 | 473.2 | -22.0% | 2.26 | no difference |
| P+ vs T | commits_per_s | 750.4 | 49.4 | 674.0 | 49.3 | 49.3 | 61.9 | -10.2% | 1.55 | no difference |
| P+ vs T | pss_mib | 43.4 | 0.01 | 2525 | 0.02 | 0.01 | 0.13 | +5716.0% | 185084 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -2032 | 133.7 | -1748 | 126.3 | 130.0 | 2484 | -14.0% | 2.19 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 319.0 | 23.2 | 1396 | 93.5 | 68.1 | 107.0 | +337.6% | 15.8 | **H3 better** |
| H3 vs T | q1_p99_us | 934.6 | 262.9 | 3706 | 891.8 | 657.4 | 657.3 | +296.5% | 4.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q2_p50_us | 331.2 | 19.9 | 1436 | 79.5 | 58.0 | 36.3 | +333.5% | 19.1 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 350.6 | 57.6 | 1979 | 185.2 | 137.1 | 144.2 | +464.4% | 11.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q1_p99_us | 1286 | 278.6 | 4489 | 1105 | 805.9 | 12844 | +249.1% | 3.98 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c2_q2_p50_us | 347.7 | 40.1 | 1985 | 190.9 | 137.9 | 94.6 | +471.0% | 11.9 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 579.8 | 40.9 | 2367 | 191.1 | 138.2 | 140.1 | +308.2% | 12.9 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 2038 | 334.2 | 6929 | 2569 | 1832 | 5771 | +239.9% | 2.67 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p50_us | 600.8 | 31.8 | 2450 | 167.0 | 120.2 | 234.5 | +307.8% | 15.4 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 2049 | 377.8 | 6893 | 2237 | 1604 | 2729 | +236.3% | 3.02 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | write_p50_us | 1458 | 148.0 | 1413 | 137.0 | 142.6 | 72.1 | -3.1% | 0.32 | BELOW FLOOR |
| H3 vs T | write_p99_us | 4457 | 646.0 | 2505 | 354.7 | 521.1 | 116.5 | -43.8% | 3.75 | **T better** |
| H3 vs T | commits_per_s | 535.3 | 120.9 | 674.0 | 49.3 | 92.3 | 23.0 | +25.9% | 1.50 | no difference |
| H3 vs T | pss_mib | 67.7 | 0.04 | 2525 | 0.02 | 0.03 | 0.25 | +3631.0% | 79176 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 13233 | 778.5 | -1748 | 126.3 | 557.7 | 10477 | -113.2% | 26.9 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 170.8 | 9.80 | 961.4 | 35.9 | 26.3 | 229.4 | +462.8% | 30.0 | **N better** |
| N vs H1 | q1_p99_us | 490.9 | 58.0 | 2825 | 239.6 | 174.3 | 490.1 | +475.5% | 13.4 | **N better** |
| N vs H1 | q2_p50_us | 9817 | 242.5 | 805.1 | 56.5 | 176.1 | 160.9 | -91.8% | 51.2 | **H1 better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2337 | 74.0 | 3913 | 2689 | 1902 | 10464 | +67.4% | 0.83 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q6_p50_us | 4367 | 358.1 | 1696 | 99.9 | 262.9 | 264.2 | -61.2% | 10.2 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 140.6 | 15.1 | 889.2 | 63.5 | 46.1 | 192.3 | +532.6% | 16.2 | **N better** |
| N vs H1 | c2_q1_p99_us | 2454 | 918.0 | 2953 | 977.9 | 948.4 | 1650 | +20.3% | 0.53 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q2_p50_us | 10040 | 347.4 | 859.9 | 56.4 | 248.9 | 2313 | -91.4% | 36.9 | **H1 better** |
| N vs H1 | c4_q1_p50_us | 156.1 | 25.2 | 863.6 | 109.4 | 79.4 | 76.7 | +453.2% | 8.91 | **N better** |
| N vs H1 | c4_q1_p99_us | 6951 | 1278 | 2645 | 692.4 | 1028 | 2785 | -61.9% | 4.19 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c4_q2_p50_us | 14017 | 3418 | 918.8 | 49.3 | 2417 | 4960 | -93.4% | 5.42 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 39671 | 3363 | 4105 | 1020 | 2485 | 5273 | -89.7% | 14.3 | **H1 better** |
| N vs H1 | write_p50_us | 600.8 | 46.5 | 1803 | 50.3 | 48.4 | 120.2 | +200.1% | 24.8 | **N better** |
| N vs H1 | write_p99_us | 1325 | 509.5 | 5819 | 1235 | 944.7 | 3433 | +339.3% | 4.76 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | commits_per_s | 1500 | 124.8 | 474.7 | 29.3 | 90.7 | 543.0 | -68.3% | 11.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | pss_mib | 25.0 | 2.53 | 287.4 | 5.97 | 4.58 | 5.60 | +1050.4% | 57.2 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 11802 | 3180 | 35423 | 2700 | 2950 | 9032 | +200.1% | 8.01 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H1 | q1_p50_us | 723.5 | 46.5 | 961.4 | 35.9 | 41.5 | 282.3 | +32.9% | 5.73 | BELOW FLOOR |
| P+ vs H1 | q1_p99_us | 2077 | 221.0 | 2825 | 239.6 | 230.5 | 519.1 | +36.0% | 3.25 | **P+ better** |
| P+ vs H1 | q2_p50_us | 580.8 | 74.1 | 805.1 | 56.5 | 65.9 | 71.8 | +38.6% | 3.40 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 383.0 | 18.9 | 929.5 | 48.2 | 36.6 | 222.7 | +142.7% | 14.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q4_p50_us | 1766 | 111.6 | 3618 | 386.8 | 284.7 | 624.0 | +104.9% | 6.51 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3798 | 320.4 | 3913 | 2689 | 1915 | 10463 | +3.0% | 0.06 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q6_p50_us | 4883 | 690.1 | 1696 | 99.9 | 493.0 | 372.9 | -65.3% | 6.46 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 972.7 | 80.0 | 889.2 | 63.5 | 72.2 | 195.3 | -8.6% | 1.16 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 6578 | 1147 | 2953 | 977.9 | 1066 | 1842 | -55.1% | 3.40 | **H1 better** |
| P+ vs H1 | c2_q2_p50_us | 781.1 | 45.2 | 859.9 | 56.4 | 51.1 | 18.8 | +10.1% | 1.54 | no difference |
| P+ vs H1 | c4_q1_p50_us | 2110 | 133.5 | 863.6 | 109.4 | 122.0 | 127.4 | -59.1% | 10.2 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 12276 | 3227 | 2645 | 692.4 | 2333 | 7395 | -78.5% | 4.13 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c4_q2_p50_us | 1925 | 122.8 | 918.8 | 49.3 | 93.6 | 184.5 | -52.3% | 10.8 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 11470 | 2528 | 4105 | 1020 | 1927 | 3782 | -64.2% | 3.82 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | write_p50_us | 1144 | 71.6 | 1803 | 50.3 | 61.8 | 159.3 | +57.7% | 10.7 | **P+ better** |
| P+ vs H1 | write_p99_us | 3210 | 262.6 | 5819 | 1235 | 892.8 | 3460 | +81.3% | 2.92 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | commits_per_s | 750.4 | 49.4 | 474.7 | 29.3 | 40.6 | 118.9 | -36.7% | 6.79 | **P+ better** |
| P+ vs H1 | pss_mib | 43.4 | 0.01 | 287.4 | 5.97 | 4.22 | 5.33 | +561.9% | 57.8 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -2032 | 133.7 | 35423 | 2700 | 1911 | 8252 | -1842.9% | 19.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 170.8 | 9.80 | 132.5 | 16.6 | 13.6 | 61.4 | -22.4% | 2.81 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q1_p99_us | 490.9 | 58.0 | 450.0 | 53.5 | 55.8 | 179.9 | -8.3% | 0.73 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 9817 | 242.5 | 228.3 | 45.0 | 174.4 | 165.3 | -97.7% | 55.0 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2337 | 74.0 | 189.0 | 28.0 | 55.9 | 204.2 | -91.9% | 38.4 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | 4367 | 358.1 | 872.7 | 146.6 | 273.6 | 231.5 | -80.0% | 12.8 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 140.6 | 15.1 | 153.9 | 34.6 | 26.7 | 171.4 | +9.5% | 0.50 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q1_p99_us | 2454 | 918.0 | 1268 | 238.6 | 670.7 | 1652 | -48.3% | 1.77 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q2_p50_us | 10040 | 347.4 | 214.6 | 71.7 | 250.8 | 2316 | -97.9% | 39.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p50_us | 156.1 | 25.2 | 248.3 | 17.3 | 21.6 | 78.5 | +59.0% | 4.27 | **N better** |
| N vs H2 | c4_q1_p99_us | 6951 | 1278 | 2777 | 198.5 | 914.8 | 3571 | -60.1% | 4.56 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p50_us | 14017 | 3418 | 340.4 | 41.6 | 2417 | 4960 | -97.6% | 5.66 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 39671 | 3363 | 2934 | 452.4 | 2399 | 5330 | -92.6% | 15.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 600.8 | 46.5 | 1602 | 113.2 | 86.5 | 183.7 | +166.6% | 11.6 | **N better** |
| N vs H2 | write_p99_us | 1325 | 509.5 | 5442 | 524.9 | 517.3 | 204.6 | +310.8% | 7.96 | **N better** |
| N vs H2 | commits_per_s | 1500 | 124.8 | 543.6 | 33.3 | 91.3 | 552.3 | -63.8% | 10.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_mib | 25.0 | 2.53 | 43.4 | 0.02 | 1.79 | 1.72 | +73.8% | 10.3 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 11802 | 3180 | -2112 | 142.6 | 2251 | 4492 | -117.9% | 6.18 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 723.5 | 46.5 | 132.5 | 16.6 | 34.9 | 175.6 | -81.7% | 16.9 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p99_us | 2077 | 221.0 | 450.0 | 53.5 | 160.8 | 248.1 | -78.3% | 10.1 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p50_us | 580.8 | 74.1 | 228.3 | 45.0 | 61.3 | 81.3 | -60.7% | 5.75 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 383.0 | 18.9 | 234.2 | 26.7 | 23.1 | 109.8 | -38.8% | 6.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q4_p50_us | 1766 | 111.6 | 307.1 | 65.5 | 91.5 | 327.0 | -82.6% | 16.0 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q5_p50_us | 3798 | 320.4 | 189.0 | 28.0 | 227.4 | 168.5 | -95.0% | 15.9 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q6_p50_us | 4883 | 690.1 | 872.7 | 146.6 | 498.8 | 350.5 | -82.1% | 8.04 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 972.7 | 80.0 | 153.9 | 34.6 | 61.6 | 174.7 | -84.2% | 13.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q1_p99_us | 6578 | 1147 | 1268 | 238.6 | 828.2 | 1845 | -80.7% | 6.41 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 781.1 | 45.2 | 214.6 | 71.7 | 59.9 | 112.0 | -72.5% | 9.45 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p50_us | 2110 | 133.5 | 248.3 | 17.3 | 95.2 | 128.5 | -88.2% | 19.6 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 12276 | 3227 | 2777 | 198.5 | 2286 | 7726 | -77.4% | 4.16 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c4_q2_p50_us | 1925 | 122.8 | 340.4 | 41.6 | 91.7 | 185.2 | -82.3% | 17.3 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 11470 | 2528 | 2934 | 452.4 | 1816 | 3861 | -74.4% | 4.70 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1144 | 71.6 | 1602 | 113.2 | 94.7 | 211.3 | +40.1% | 4.84 | **P+ better** |
| P+ vs H2 | write_p99_us | 3210 | 262.6 | 5442 | 524.9 | 415.0 | 480.4 | +69.5% | 5.38 | **P+ better** |
| P+ vs H2 | commits_per_s | 750.4 | 49.4 | 543.6 | 33.3 | 42.1 | 156.0 | -27.6% | 4.91 | **P+ better** |
| P+ vs H2 | pss_mib | 43.4 | 0.01 | 43.4 | 0.02 | 0.01 | 0.07 | -0.0% | 0.14 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | -2032 | 133.7 | -2112 | 142.6 | 138.2 | 2588 | +3.9% | 0.58 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 184.1 | 12.6 | 736.0 | 30.6 | 23.4 | 96.3 | +299.9% | 23.6 | **N better** |
| N vs P+ | q1_p99_us | 669.0 | 192.8 | 2393 | 548.1 | 410.8 | 525.4 | +257.8% | 4.20 | **N better** |
| N vs P+ | q2_p50_us | 9603 | 312.1 | 578.1 | 43.7 | 222.8 | 332.8 | -94.0% | 40.5 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 1987 | 417.1 | 3129 | 989.5 | 759.3 | 2025 | +57.5% | 1.50 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 147.2 | 18.8 | 972.2 | 37.4 | 29.6 | 44.3 | +560.7% | 27.9 | **N better** |
| N vs P+ | c2_q1_p99_us | 3045 | 1546 | 7052 | 1332 | 1443 | 1232 | +131.6% | 2.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 10430 | 792.2 | 801.7 | 44.8 | 561.1 | 2354 | -92.3% | 17.2 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 180.7 | 20.7 | 2018 | 125.6 | 90.0 | 146.8 | +1016.7% | 20.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 7922 | 1239 | 507765 | 496291 | 350932 | 2859 | +6309.4% | 1.42 | no difference |
| N vs P+ | c4_q2_p50_us | 17493 | 937.7 | 1766 | 104.3 | 667.1 | 666.2 | -89.9% | 23.6 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 42488 | 5564 | 14308 | 5595 | 5579 | 8233 | -66.3% | 5.05 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 616.8 | 45.7 | 1170 | 104.1 | 80.4 | 76.8 | +89.6% | 6.87 | **N better** |
| N vs P+ | write_p99_us | 1744 | 625.3 | 3259 | 177.2 | 459.6 | 545.0 | +86.8% | 3.30 | **N better** |
| N vs P+ | commits_per_s | 1419 | 143.7 | 749.8 | 50.9 | 107.8 | 102.4 | -47.2% | 6.21 | **N better** |
| N vs P+ | pss_mib | 25.1 | 3.09 | 43.3 | 0.02 | 2.18 | 0.22 | +72.6% | 8.34 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 18158 | 1803 | -2076 | 140.1 | 1279 | 8406 | -111.4% | 15.8 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 184.1 | 12.6 | 142.8 | 8.97 | 10.9 | 21.9 | -22.4% | 3.78 | **P better** |
| N vs P | q1_p99_us | 669.0 | 192.8 | 813.9 | 366.6 | 292.9 | 1366 | +21.7% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 9603 | 312.1 | 223.6 | 23.7 | 221.3 | 307.0 | -97.7% | 42.4 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 1987 | 417.1 | 209.9 | 34.7 | 295.9 | 1431 | -89.4% | 6.00 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 147.2 | 18.8 | 163.2 | 20.6 | 19.7 | 96.3 | +10.9% | 0.81 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 3045 | 1546 | 2067 | 262.8 | 1109 | 1371 | -32.1% | 0.88 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 10430 | 792.2 | 266.1 | 46.6 | 561.1 | 2349 | -97.4% | 18.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 180.7 | 20.7 | 214.0 | 45.0 | 35.0 | 34.7 | +18.4% | 0.95 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 7922 | 1239 | 2469 | 350.7 | 910.3 | 1957 | -68.8% | 5.99 | **P better** |
| N vs P | c4_q2_p50_us | 17493 | 937.7 | 356.9 | 43.7 | 663.8 | 562.8 | -98.0% | 25.8 | **P better** |
| N vs P | c4_q2_p99_us | 42488 | 5564 | 3036 | 666.1 | 3962 | 7738 | -92.9% | 9.96 | **P better** |
| N vs P | write_p50_us | 616.8 | 45.7 | 19176 | 133.3 | 99.6 | 198.1 | +3008.8% | 186.3 | **N better** |
| N vs P | write_p99_us | 1744 | 625.3 | 32770 | 3581 | 2570 | 1495 | +1778.7% | 12.1 | **N better** |
| N vs P | commits_per_s | 1419 | 143.7 | 47.5 | 1.94 | 101.6 | 18.1 | -96.7% | 13.5 | **N better** |
| N vs P | pss_mib | 25.1 | 3.09 | 43.3 | 0.01 | 2.18 | 0.03 | +72.7% | 8.35 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 18158 | 1803 | -2063 | 136.3 | 1279 | 5354 | -111.4% | 15.8 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 736.0 | 30.6 | 142.8 | 8.97 | 22.5 | 98.5 | -80.6% | 26.3 | **P better** |
| P+ vs P | q1_p99_us | 2393 | 548.1 | 813.9 | 366.6 | 466.3 | 1461 | -66.0% | 3.39 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 578.1 | 43.7 | 223.6 | 23.7 | 35.2 | 131.0 | -61.3% | 10.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 348.7 | 42.8 | 275.3 | 36.0 | 39.6 | 144.4 | -21.0% | 1.85 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 1760 | 196.8 | 285.3 | 14.7 | 139.5 | 239.4 | -83.8% | 10.6 | **P better** |
| P+ vs P | q5_p50_us | 3129 | 989.5 | 209.9 | 34.7 | 700.1 | 1447 | -93.3% | 4.17 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 972.2 | 37.4 | 163.2 | 20.6 | 30.2 | 105.9 | -83.2% | 26.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 7052 | 1332 | 2067 | 262.8 | 960.1 | 724.4 | -70.7% | 5.19 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 801.7 | 44.8 | 266.1 | 46.6 | 45.7 | 218.6 | -66.8% | 11.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 2018 | 125.6 | 214.0 | 45.0 | 94.3 | 144.1 | -89.4% | 19.1 | **P better** |
| P+ vs P | c4_q1_p99_us | 507765 | 496291 | 2469 | 350.7 | 350931 | 2091 | -99.5% | 1.44 | no difference |
| P+ vs P | c4_q2_p50_us | 1766 | 104.3 | 356.9 | 43.7 | 80.0 | 356.7 | -79.8% | 17.6 | **P better** |
| P+ vs P | c4_q2_p99_us | 14308 | 5595 | 3036 | 666.1 | 3984 | 2926 | -78.8% | 2.83 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 1170 | 104.1 | 19176 | 133.3 | 119.6 | 212.0 | +1539.6% | 150.5 | **P+ better** |
| P+ vs P | write_p99_us | 3259 | 177.2 | 32770 | 3581 | 2535 | 1566 | +905.6% | 11.6 | **P+ better** |
| P+ vs P | commits_per_s | 749.8 | 50.9 | 47.5 | 1.94 | 36.0 | 100.8 | -93.7% | 19.5 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.02 | 43.3 | 0.01 | 0.02 | 0.22 | +0.0% | 0.78 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2076 | 140.1 | -2063 | 136.3 | 138.2 | 7027 | -0.7% | 0.10 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 142.8 | 8.97 | 144.3 | 8.03 | 8.51 | 21.5 | +1.1% | 0.18 | BELOW FLOOR |
| P vs M | q1_p99_us | 813.9 | 366.6 | 1095 | 358.9 | 362.8 | 1393 | +34.5% | 0.77 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 223.6 | 23.7 | 242.9 | 26.2 | 25.0 | 36.4 | +8.7% | 0.77 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 275.3 | 36.0 | 283.0 | 74.2 | 58.3 | 107.9 | +2.8% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q4_p50_us | 285.3 | 14.7 | 308.9 | 31.4 | 24.5 | 129.3 | +8.3% | 0.96 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q5_p50_us | 209.9 | 34.7 | 178.9 | 24.9 | 30.2 | 147.2 | -14.8% | 1.03 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 163.2 | 20.6 | 132.2 | 20.9 | 20.8 | 154.3 | -19.0% | 1.49 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 2067 | 262.8 | 2057 | 182.8 | 226.4 | 701.8 | -0.5% | 0.05 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 266.1 | 46.6 | 216.8 | 29.0 | 38.8 | 142.3 | -18.5% | 1.27 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 214.0 | 45.0 | 238.9 | 49.3 | 47.2 | 55.5 | +11.7% | 0.53 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2469 | 350.7 | 2845 | 215.0 | 290.9 | 2550 | +15.2% | 1.29 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 356.9 | 43.7 | 334.0 | 48.0 | 45.9 | 12.0 | -6.4% | 0.50 | no difference |
| P vs M | c4_q2_p99_us | 3036 | 666.1 | 3291 | 727.2 | 697.4 | 958.9 | +8.4% | 0.37 | BELOW FLOOR |
| P vs M | write_p50_us | 19176 | 133.3 | 19410 | 415.8 | 308.7 | 2145 | +1.2% | 0.76 | BELOW FLOOR |
| P vs M | write_p99_us | 32770 | 3581 | 31225 | 1522 | 2751 | 7203 | -4.7% | 0.56 | BELOW FLOOR |
| P vs M | commits_per_s | 47.5 | 1.94 | 48.1 | 0.91 | 1.52 | 2.53 | +1.2% | 0.38 | BELOW FLOOR |
| P vs M | pss_mib | 43.3 | 0.01 | 43.5 | 0.01 | 0.01 | 0.10 | +0.5% | 16.5 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2063 | 136.3 | -1994 | 131.4 | 133.9 | 2763 | -3.3% | 0.51 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 184.1 | 12.6 | 332.7 | 17.1 | 15.0 | 67.8 | +80.8% | 9.91 | **N better** |
| N vs H3 | q1_p99_us | 669.0 | 192.8 | 1017 | 265.6 | 232.1 | 245.0 | +52.0% | 1.50 | no difference |
| N vs H3 | q2_p50_us | 9603 | 312.1 | 375.3 | 29.4 | 221.7 | 308.4 | -96.1% | 41.6 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 1987 | 417.1 | 3432 | 681.0 | 564.7 | 1552 | +72.7% | 2.56 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 147.2 | 18.8 | 368.7 | 15.7 | 17.3 | 116.1 | +150.6% | 12.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 3045 | 1546 | 1346 | 321.7 | 1116 | 1279 | -55.8% | 1.52 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 10430 | 792.2 | 381.0 | 32.5 | 560.6 | 2346 | -96.3% | 17.9 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 180.7 | 20.7 | 552.5 | 46.2 | 35.8 | 204.5 | +205.8% | 10.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 7922 | 1239 | 2044 | 253.0 | 893.9 | 2808 | -74.2% | 6.58 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 17493 | 937.7 | 613.2 | 48.6 | 663.9 | 600.2 | -96.5% | 25.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 42488 | 5564 | 2615 | 755.1 | 3970 | 8101 | -93.8% | 10.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 616.8 | 45.7 | 1512 | 81.3 | 66.0 | 518.3 | +145.2% | 13.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p99_us | 1744 | 625.3 | 5582 | 1571 | 1196 | 3966 | +220.0% | 3.21 | BELOW FLOOR |
| N vs H3 | commits_per_s | 1419 | 143.7 | 535.7 | 117.8 | 131.4 | 211.4 | -62.3% | 6.73 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | pss_mib | 25.1 | 3.09 | 67.3 | 0.05 | 2.18 | 0.04 | +168.2% | 19.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 18158 | 1803 | 13121 | 939.9 | 1438 | 12355 | -27.7% | 3.50 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 736.0 | 30.6 | 332.7 | 17.1 | 24.8 | 117.6 | -54.8% | 16.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2393 | 548.1 | 1017 | 265.6 | 430.7 | 571.7 | -57.5% | 3.20 | **H3 better** |
| P+ vs H3 | q2_p50_us | 578.1 | 43.7 | 375.3 | 29.4 | 37.3 | 134.2 | -35.1% | 5.44 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 348.7 | 42.8 | 386.5 | 50.0 | 46.6 | 99.0 | +10.8% | 0.81 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1760 | 196.8 | 1942 | 229.8 | 213.9 | 453.0 | +10.3% | 0.85 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3129 | 989.5 | 3432 | 681.0 | 849.4 | 1567 | +9.7% | 0.36 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 972.2 | 37.4 | 368.7 | 15.7 | 28.7 | 124.2 | -62.1% | 21.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 7052 | 1332 | 1346 | 321.7 | 969.0 | 529.8 | -80.9% | 5.89 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 801.7 | 44.8 | 381.0 | 32.5 | 39.2 | 188.4 | -52.5% | 10.7 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2018 | 125.6 | 552.5 | 46.2 | 94.6 | 247.7 | -72.6% | 15.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 507765 | 496291 | 2044 | 253.0 | 350931 | 2902 | -99.6% | 1.44 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1766 | 104.3 | 613.2 | 48.6 | 81.4 | 413.1 | -65.3% | 14.2 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 14308 | 5595 | 2615 | 755.1 | 3992 | 3783 | -81.7% | 2.93 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1170 | 104.1 | 1512 | 81.3 | 93.4 | 523.7 | +29.3% | 3.67 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p99_us | 3259 | 177.2 | 5582 | 1571 | 1118 | 3993 | +71.3% | 2.08 | BELOW FLOOR |
| P+ vs H3 | commits_per_s | 749.8 | 50.9 | 535.7 | 117.8 | 90.7 | 233.5 | -28.5% | 2.36 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | pss_mib | 43.3 | 0.02 | 67.3 | 0.05 | 0.04 | 0.22 | +55.4% | 599.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2076 | 140.1 | 13121 | 939.9 | 672.0 | 13166 | -731.9% | 22.6 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 184.1 | 12.6 | 1336 | 88.8 | 63.4 | 175.9 | +625.9% | 18.2 | **N better** |
| N vs T | q1_p99_us | 669.0 | 192.8 | 3980 | 1015 | 730.8 | 77.3 | +494.9% | 4.53 | **N better** |
| N vs T | q2_p50_us | 9603 | 312.1 | 1328 | 72.5 | 226.6 | 309.7 | -86.2% | 36.5 | **T better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 147.2 | 18.8 | 1830 | 45.9 | 35.1 | 20.8 | +1143.2% | 48.0 | **N better** |
| N vs T | c2_q1_p99_us | 3045 | 1546 | 4668 | 893.0 | 1262 | 2213 | +53.3% | 1.29 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c2_q2_p50_us | 10430 | 792.2 | 1969 | 68.8 | 562.3 | 2347 | -81.1% | 15.0 | **T better** |
| N vs T | c4_q1_p50_us | 180.7 | 20.7 | 2131 | 116.4 | 83.6 | 49.0 | +1079.3% | 23.3 | **N better** |
| N vs T | c4_q1_p99_us | 7922 | 1239 | 6636 | 1780 | 1533 | 3787 | -16.2% | 0.84 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c4_q2_p50_us | 17493 | 937.7 | 2219 | 90.9 | 666.1 | 565.7 | -87.3% | 22.9 | **T better** |
| N vs T | c4_q2_p99_us | 42488 | 5564 | 6260 | 1691 | 4112 | 7728 | -85.3% | 8.81 | **T better** |
| N vs T | write_p50_us | 616.8 | 45.7 | 1279 | 88.1 | 70.2 | 11.6 | +107.3% | 9.43 | **N better** |
| N vs T | write_p99_us | 1744 | 625.3 | 2958 | 535.3 | 582.0 | 1371 | +69.6% | 2.08 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | commits_per_s | 1419 | 143.7 | 689.3 | 46.0 | 106.7 | 54.0 | -51.4% | 6.84 | **N better** |
| N vs T | pss_mib | 25.1 | 3.09 | 2526 | 0.03 | 2.18 | 1.14 | +9974.5% | 1145 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 18158 | 1803 | -3227 | 240.0 | 1286 | 5075 | -117.8% | 16.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs T | q1_p50_us | 736.0 | 30.6 | 1336 | 88.8 | 66.4 | 200.4 | +81.6% | 9.04 | **P+ better** |
| P+ vs T | q1_p99_us | 2393 | 548.1 | 3980 | 1015 | 815.9 | 522.3 | +66.3% | 1.94 | no difference |
| P+ vs T | q2_p50_us | 578.1 | 43.7 | 1328 | 72.5 | 59.9 | 137.2 | +129.8% | 12.5 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 972.2 | 37.4 | 1830 | 45.9 | 41.9 | 48.8 | +88.2% | 20.5 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 7052 | 1332 | 4668 | 893.0 | 1134 | 1882 | -33.8% | 2.10 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c2_q2_p50_us | 801.7 | 44.8 | 1969 | 68.8 | 58.1 | 202.0 | +145.5% | 20.1 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2018 | 125.6 | 2131 | 116.4 | 121.0 | 148.2 | +5.6% | 0.93 | BELOW FLOOR |
| P+ vs T | c4_q1_p99_us | 507765 | 496291 | 6636 | 1780 | 350933 | 3858 | -98.7% | 1.43 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c4_q2_p50_us | 1766 | 104.3 | 2219 | 90.9 | 97.8 | 361.2 | +25.6% | 4.63 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 14308 | 5595 | 6260 | 1691 | 4133 | 2899 | -56.2% | 1.95 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | write_p50_us | 1170 | 104.1 | 1279 | 88.1 | 96.4 | 76.3 | +9.3% | 1.13 | no difference |
| P+ vs T | write_p99_us | 3259 | 177.2 | 2958 | 535.3 | 398.7 | 1448 | -9.2% | 0.76 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 749.8 | 50.9 | 689.3 | 46.0 | 48.5 | 112.9 | -8.1% | 1.25 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.02 | 2526 | 0.03 | 0.03 | 1.16 | +5736.5% | 93762 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -2076 | 140.1 | -3227 | 240.0 | 196.5 | 6817 | +55.4% | 5.85 | REFUSED (warm-up MAD above 15% of the median on P+) |
| H3 vs T | q1_p50_us | 332.7 | 17.1 | 1336 | 88.8 | 63.9 | 188.4 | +301.6% | 15.7 | **H3 better** |
| H3 vs T | q1_p99_us | 1017 | 265.6 | 3980 | 1015 | 742.2 | 238.3 | +291.5% | 3.99 | **H3 better** |
| H3 vs T | q2_p50_us | 375.3 | 29.4 | 1328 | 72.5 | 55.3 | 56.2 | +254.0% | 17.2 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 368.7 | 15.7 | 1830 | 45.9 | 34.3 | 117.9 | +396.2% | 42.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q1_p99_us | 1346 | 321.7 | 4668 | 893.0 | 671.2 | 1913 | +246.7% | 4.95 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c2_q2_p50_us | 381.0 | 32.5 | 1969 | 68.8 | 53.8 | 72.9 | +416.6% | 29.5 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 552.5 | 46.2 | 2131 | 116.4 | 88.5 | 205.5 | +285.7% | 17.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 2044 | 253.0 | 6636 | 1780 | 1271 | 3820 | +224.6% | 3.61 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c4_q2_p50_us | 613.2 | 48.6 | 2219 | 90.9 | 72.9 | 216.4 | +261.8% | 22.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p99_us | 2615 | 755.1 | 6260 | 1691 | 1310 | 2498 | +139.4% | 2.78 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | write_p50_us | 1512 | 81.3 | 1279 | 88.1 | 84.8 | 518.2 | -15.5% | 2.76 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | write_p99_us | 5582 | 1571 | 2958 | 535.3 | 1174 | 4186 | -47.0% | 2.24 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 535.7 | 117.8 | 689.3 | 46.0 | 89.4 | 216.7 | +28.7% | 1.72 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | pss_mib | 67.3 | 0.05 | 2526 | 0.03 | 0.04 | 1.14 | +3655.8% | 59129 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 13121 | 939.9 | -3227 | 240.0 | 685.9 | 11334 | -124.6% | 23.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H1 | q1_p50_us | 184.1 | 12.6 | 946.4 | 56.3 | 40.8 | 89.0 | +414.2% | 18.7 | **N better** |
| N vs H1 | q1_p99_us | 669.0 | 192.8 | 2658 | 553.3 | 414.3 | 841.5 | +297.3% | 4.80 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q2_p50_us | 9603 | 312.1 | 878.8 | 65.1 | 225.4 | 307.3 | -90.8% | 38.7 | **H1 better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 1987 | 417.1 | 5588 | 2641 | 1890 | 6570 | +181.2% | 1.90 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | c2_q1_p50_us | 147.2 | 18.8 | 860.8 | 57.6 | 42.8 | 95.3 | +485.0% | 16.7 | **N better** |
| N vs H1 | c2_q1_p99_us | 3045 | 1546 | 2474 | 406.8 | 1130 | 1247 | -18.8% | 0.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q2_p50_us | 10430 | 792.2 | 824.6 | 43.9 | 561.0 | 2347 | -92.1% | 17.1 | **H1 better** |
| N vs H1 | c4_q1_p50_us | 180.7 | 20.7 | 810.0 | 49.9 | 38.2 | 43.6 | +348.2% | 16.5 | **N better** |
| N vs H1 | c4_q1_p99_us | 7922 | 1239 | 3635 | 946.4 | 1102 | 2009 | -54.1% | 3.89 | **H1 better** |
| N vs H1 | c4_q2_p50_us | 17493 | 937.7 | 857.7 | 57.6 | 664.3 | 562.8 | -95.1% | 25.0 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 42488 | 5564 | 3211 | 478.5 | 3949 | 7724 | -92.4% | 9.95 | **H1 better** |
| N vs H1 | write_p50_us | 616.8 | 45.7 | 1753 | 111.4 | 85.1 | 464.7 | +184.2% | 13.3 | **N better** |
| N vs H1 | write_p99_us | 1744 | 625.3 | 5369 | 766.5 | 699.5 | 2745 | +207.8% | 5.18 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | commits_per_s | 1419 | 143.7 | 494.0 | 27.0 | 103.4 | 117.7 | -65.2% | 8.95 | **N better** |
| N vs H1 | pss_mib | 25.1 | 3.09 | 284.4 | 5.26 | 4.31 | 19.5 | +1034.3% | 60.1 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 18158 | 1803 | 30438 | 2360 | 2100 | 20694 | +67.6% | 5.85 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| P+ vs H1 | q1_p50_us | 736.0 | 30.6 | 946.4 | 56.3 | 45.3 | 130.9 | +28.6% | 4.65 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2393 | 548.1 | 2658 | 553.3 | 550.7 | 987.4 | +11.1% | 0.48 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q2_p50_us | 578.1 | 43.7 | 878.8 | 65.1 | 55.4 | 131.7 | +52.0% | 5.42 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 348.7 | 42.8 | 896.2 | 106.5 | 81.2 | 133.3 | +157.0% | 6.74 | **P+ better** |
| P+ vs H1 | q4_p50_us | 1760 | 196.8 | 3870 | 367.9 | 295.0 | 325.1 | +119.8% | 7.15 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3129 | 989.5 | 5588 | 2641 | 1994 | 6574 | +78.6% | 1.23 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | c2_q1_p50_us | 972.2 | 37.4 | 860.8 | 57.6 | 48.6 | 105.0 | -11.5% | 2.29 | no difference |
| P+ vs H1 | c2_q1_p99_us | 7052 | 1332 | 2474 | 406.8 | 984.9 | 445.6 | -64.9% | 4.65 | **H1 better** |
| P+ vs H1 | c2_q2_p50_us | 801.7 | 44.8 | 824.6 | 43.9 | 44.3 | 193.5 | +2.9% | 0.52 | BELOW FLOOR |
| P+ vs H1 | c4_q1_p50_us | 2018 | 125.6 | 810.0 | 49.9 | 95.5 | 146.5 | -59.9% | 12.6 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 507765 | 496291 | 3635 | 946.4 | 350931 | 2140 | -99.3% | 1.44 | no difference |
| P+ vs H1 | c4_q2_p50_us | 1766 | 104.3 | 857.7 | 57.6 | 84.3 | 356.7 | -51.4% | 10.8 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 14308 | 5595 | 3211 | 478.5 | 3971 | 2890 | -77.6% | 2.79 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | write_p50_us | 1170 | 104.1 | 1753 | 111.4 | 107.8 | 470.8 | +49.9% | 5.41 | **P+ better** |
| P+ vs H1 | write_p99_us | 3259 | 177.2 | 5369 | 766.5 | 556.3 | 2784 | +64.7% | 3.79 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | commits_per_s | 749.8 | 50.9 | 494.0 | 27.0 | 40.8 | 153.9 | -34.1% | 6.28 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.02 | 284.4 | 5.26 | 3.72 | 19.5 | +557.2% | 64.8 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -2076 | 140.1 | 30438 | 2360 | 1672 | 21188 | -1565.9% | 19.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 184.1 | 12.6 | 129.8 | 7.51 | 10.4 | 40.6 | -29.5% | 5.24 | **H2 better** |
| N vs H2 | q1_p99_us | 669.0 | 192.8 | 434.7 | 62.6 | 143.3 | 270.7 | -35.0% | 1.63 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 9603 | 312.1 | 226.3 | 10.7 | 220.8 | 316.6 | -97.6% | 42.5 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 1987 | 417.1 | 175.4 | 21.4 | 295.3 | 1424 | -91.2% | 6.13 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 147.2 | 18.8 | 163.3 | 42.2 | 32.7 | 56.9 | +11.0% | 0.49 | BELOW FLOOR |
| N vs H2 | c2_q1_p99_us | 3045 | 1546 | 1899 | 466.4 | 1142 | 1310 | -37.6% | 1.00 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q2_p50_us | 10430 | 792.2 | 286.5 | 44.1 | 561.0 | 2346 | -97.3% | 18.1 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 180.7 | 20.7 | 217.5 | 27.5 | 24.3 | 73.5 | +20.4% | 1.51 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p99_us | 7922 | 1239 | 2628 | 505.4 | 945.9 | 2020 | -66.8% | 5.60 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 17493 | 937.7 | 360.2 | 31.1 | 663.4 | 563.2 | -97.9% | 25.8 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 42488 | 5564 | 3686 | 850.7 | 3980 | 7882 | -91.3% | 9.75 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 616.8 | 45.7 | 1516 | 48.8 | 47.3 | 525.2 | +145.8% | 19.0 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p99_us | 1744 | 625.3 | 5517 | 456.3 | 547.4 | 364.0 | +216.3% | 6.89 | **N better** |
| N vs H2 | commits_per_s | 1419 | 143.7 | 558.6 | 36.6 | 104.9 | 156.9 | -60.6% | 8.21 | **N better** |
| N vs H2 | pss_mib | 25.1 | 3.09 | 43.4 | 0.01 | 2.18 | 0.04 | +72.9% | 8.37 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 18158 | 1803 | -2234 | 157.2 | 1280 | 5411 | -112.3% | 15.9 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 736.0 | 30.6 | 129.8 | 7.51 | 22.2 | 104.3 | -82.4% | 27.2 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2393 | 548.1 | 434.7 | 62.6 | 390.1 | 583.2 | -81.8% | 5.02 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p50_us | 578.1 | 43.7 | 226.3 | 10.7 | 31.8 | 152.2 | -60.9% | 11.1 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 348.7 | 42.8 | 228.1 | 31.5 | 37.6 | 106.9 | -34.6% | 3.21 | **H2 better** |
| P+ vs H2 | q4_p50_us | 1760 | 196.8 | 256.9 | 15.7 | 139.6 | 231.3 | -85.4% | 10.8 | **H2 better** |
| P+ vs H2 | q5_p50_us | 3129 | 989.5 | 175.4 | 21.4 | 699.9 | 1441 | -94.4% | 4.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 972.2 | 37.4 | 163.3 | 42.2 | 39.9 | 72.1 | -83.2% | 20.3 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 7052 | 1332 | 1899 | 466.4 | 998.0 | 600.4 | -73.1% | 5.16 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 801.7 | 44.8 | 286.5 | 44.1 | 44.5 | 188.8 | -64.3% | 11.6 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 2018 | 125.6 | 217.5 | 27.5 | 90.9 | 158.0 | -89.2% | 19.8 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 507765 | 496291 | 2628 | 505.4 | 350931 | 2150 | -99.5% | 1.44 | no difference |
| P+ vs H2 | c4_q2_p50_us | 1766 | 104.3 | 360.2 | 31.1 | 77.0 | 357.3 | -79.6% | 18.3 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 14308 | 5595 | 3686 | 850.7 | 4002 | 3287 | -74.2% | 2.65 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1170 | 104.1 | 1516 | 48.8 | 81.3 | 530.6 | +29.6% | 4.26 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | write_p99_us | 3259 | 177.2 | 5517 | 456.3 | 346.1 | 590.4 | +69.3% | 6.53 | **P+ better** |
| P+ vs H2 | commits_per_s | 749.8 | 50.9 | 558.6 | 36.6 | 44.3 | 185.6 | -25.5% | 4.31 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.02 | 43.4 | 0.01 | 0.02 | 0.22 | +0.2% | 4.08 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | -2076 | 140.1 | -2234 | 157.2 | 148.9 | 7070 | +7.6% | 1.06 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 181.6 | 12.4 | 717.9 | 38.7 | 28.7 | 179.2 | +295.3% | 18.7 | **N better** |
| N vs P+ | q1_p99_us | 587.1 | 46.8 | 2567 | 678.2 | 480.7 | 1425 | +337.2% | 4.12 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 74786 | 7720 | 567.5 | 22.3 | 5459 | 5324 | -99.2% | 13.6 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 13967 | 5627 | 28528 | 4761 | 5212 | 3450 | +104.2% | 2.79 | no difference |
| N vs P+ | q6_p50_us | 54925 | 641.6 | 61881 | 3072 | 2219 | 4136 | +12.7% | 3.14 | **N better** |
| N vs P+ | c2_q1_p50_us | 149.6 | 8.08 | 959.7 | 80.4 | 57.1 | 290.1 | +541.6% | 14.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q1_p99_us | 3645 | 1124 | 7614 | 1296 | 1213 | 3083 | +108.9% | 3.27 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 93787 | 14248 | 728.9 | 112.9 | 10075 | 4401 | -99.2% | 9.24 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 173.4 | 18.9 | 2005 | 102.2 | 73.5 | 95.0 | +1056.4% | 24.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 8292 | 1404 | 10796 | 832.1 | 1154 | 4809 | +30.2% | 2.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 118889 | 6270 | 1834 | 144.4 | 4435 | 11958 | -98.5% | 26.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 384623 | 28347 | 10553 | 2249 | 20107 | 47760 | -97.3% | 18.6 | **P+ better** |
| N vs P+ | write_p50_us | 514.6 | 35.5 | 1093 | 35.6 | 35.6 | 42.3 | +112.5% | 16.3 | **N better** |
| N vs P+ | write_p99_us | 1016 | 218.6 | 3808 | 471.6 | 367.5 | 2917 | +274.6% | 7.59 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 1724 | 86.1 | 750.1 | 51.3 | 70.8 | 224.6 | -56.5% | 13.7 | **N better** |
| N vs P+ | pss_mib | 209.6 | 25.0 | 43.3 | 0.06 | 17.7 | 105.4 | -79.3% | 9.40 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_growth_bytes_per_key_read | 59104 | 4618 | -757.9 | 518.4 | 3286 | 7729 | -101.3% | 18.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 181.6 | 12.4 | 123.9 | 9.39 | 11.0 | 50.3 | -31.8% | 5.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q1_p99_us | 587.1 | 46.8 | 618.4 | 39.7 | 43.4 | 26.9 | +5.3% | 0.72 | no difference |
| N vs P | q2_p50_us | 74786 | 7720 | 203.4 | 32.0 | 5459 | 5324 | -99.7% | 13.7 | **P better** |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 13967 | 5627 | 172.7 | 28.2 | 3979 | 211.0 | -98.8% | 3.47 | **P better** |
| N vs P | q6_p50_us | 54925 | 641.6 | 4261 | 449.1 | 553.8 | 3523 | -92.2% | 91.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p50_us | 149.6 | 8.08 | 186.2 | 27.6 | 20.4 | 109.7 | +24.5% | 1.80 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 3645 | 1124 | 2650 | 561.5 | 888.5 | 1616 | -27.3% | 1.12 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 93787 | 14248 | 337.7 | 30.0 | 10075 | 4406 | -99.6% | 9.28 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 173.4 | 18.9 | 188.4 | 38.4 | 30.2 | 78.9 | +8.6% | 0.50 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 8292 | 1404 | 2840 | 374.8 | 1027 | 3463 | -65.7% | 5.31 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 118889 | 6270 | 353.3 | 46.0 | 4433 | 11950 | -99.7% | 26.7 | **P better** |
| N vs P | c4_q2_p99_us | 384623 | 28347 | 3922 | 415.4 | 20046 | 47759 | -99.0% | 19.0 | **P better** |
| N vs P | write_p50_us | 514.6 | 35.5 | 189823 | 8131 | 5750 | 11767 | +36789.6% | 32.9 | **N better** |
| N vs P | write_p99_us | 1016 | 218.6 | 255304 | 13768 | 9736 | 6415 | +25020.1% | 26.1 | **N better** |
| N vs P | commits_per_s | 1724 | 86.1 | 5.09 | 0.15 | 60.9 | 208.0 | -99.7% | 28.2 | **N better** |
| N vs P | pss_mib | 209.6 | 25.0 | 43.4 | 0.04 | 17.7 | 105.4 | -79.3% | 9.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_growth_bytes_per_key_read | 59104 | 4618 | -713.8 | 504.4 | 3285 | 7716 | -101.2% | 18.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 717.9 | 38.7 | 123.9 | 9.39 | 28.2 | 184.8 | -82.7% | 21.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p99_us | 2567 | 678.2 | 618.4 | 39.7 | 480.4 | 1425 | -75.9% | 4.06 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 567.5 | 22.3 | 203.4 | 32.0 | 27.6 | 72.1 | -64.2% | 13.2 | **P better** |
| P+ vs P | q3_p50_us | 340.0 | 51.7 | 235.3 | 34.0 | 43.8 | 207.9 | -30.8% | 2.39 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q4_p50_us | 16398 | 1125 | 1015 | 21.7 | 795.5 | 124.6 | -93.8% | 19.3 | **P better** |
| P+ vs P | q5_p50_us | 28528 | 4761 | 172.7 | 28.2 | 3366 | 3444 | -99.4% | 8.42 | **P better** |
| P+ vs P | q6_p50_us | 61881 | 3072 | 4261 | 449.1 | 2195 | 5430 | -93.1% | 26.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p50_us | 959.7 | 80.4 | 186.2 | 27.6 | 60.1 | 301.8 | -80.6% | 12.9 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q1_p99_us | 7614 | 1296 | 2650 | 561.5 | 998.7 | 2941 | -65.2% | 4.97 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 728.9 | 112.9 | 337.7 | 30.0 | 82.6 | 249.1 | -53.7% | 4.73 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 2005 | 102.2 | 188.4 | 38.4 | 77.2 | 64.0 | -90.6% | 23.5 | **P better** |
| P+ vs P | c4_q1_p99_us | 10796 | 832.1 | 2840 | 374.8 | 645.3 | 3339 | -73.7% | 12.3 | **P better** |
| P+ vs P | c4_q2_p50_us | 1834 | 144.4 | 353.3 | 46.0 | 107.1 | 461.0 | -80.7% | 13.8 | **P better** |
| P+ vs P | c4_q2_p99_us | 10553 | 2249 | 3922 | 415.4 | 1618 | 1567 | -62.8% | 4.10 | **P better** |
| P+ vs P | write_p50_us | 1093 | 35.6 | 189823 | 8131 | 5750 | 11766 | +17261.0% | 32.8 | **P+ better** |
| P+ vs P | write_p99_us | 3808 | 471.6 | 255304 | 13768 | 9741 | 7043 | +6605.3% | 25.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 750.1 | 51.3 | 5.09 | 0.15 | 36.2 | 84.8 | -99.3% | 20.6 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.06 | 43.4 | 0.04 | 0.05 | 0.10 | +0.2% | 1.75 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -757.9 | 518.4 | -713.8 | 504.4 | 511.5 | 2513 | -5.8% | 0.09 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 123.9 | 9.39 | 90.8 | 22.3 | 17.1 | 63.0 | -26.7% | 1.93 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q1_p99_us | 618.4 | 39.7 | 561.0 | 76.7 | 61.0 | 293.0 | -9.3% | 0.94 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 203.4 | 32.0 | 170.1 | 17.6 | 25.8 | 45.6 | -16.4% | 1.29 | BELOW FLOOR |
| P vs M | q3_p50_us | 235.3 | 34.0 | 230.8 | 54.1 | 45.2 | 9.18 | -1.9% | 0.10 | BELOW FLOOR |
| P vs M | q4_p50_us | 1015 | 21.7 | 1010 | 47.3 | 36.8 | 120.4 | -0.5% | 0.14 | BELOW FLOOR |
| P vs M | q5_p50_us | 172.7 | 28.2 | 150.2 | 25.9 | 27.1 | 79.7 | -13.0% | 0.83 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | 4261 | 449.1 | 5938 | 169.1 | 339.4 | 3863 | +39.3% | 4.94 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p50_us | 186.2 | 27.6 | 90.7 | 37.7 | 33.1 | 101.7 | -51.3% | 2.89 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 2650 | 561.5 | 1827 | 506.0 | 534.5 | 1281 | -31.1% | 1.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 337.7 | 30.0 | 252.2 | 33.5 | 31.8 | 232.4 | -25.3% | 2.69 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 188.4 | 38.4 | 166.0 | 28.8 | 33.9 | 81.2 | -11.9% | 0.66 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2840 | 374.8 | 2684 | 281.4 | 331.4 | 206.8 | -5.5% | 0.47 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 353.3 | 46.0 | 366.9 | 73.0 | 61.0 | 107.8 | +3.8% | 0.22 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3922 | 415.4 | 4927 | 1091 | 825.5 | 1089 | +25.6% | 1.22 | BELOW FLOOR |
| P vs M | write_p50_us | 189823 | 8131 | 190182 | 2965 | 6120 | 13597 | +0.2% | 0.06 | BELOW FLOOR |
| P vs M | write_p99_us | 255304 | 13768 | 244712 | 9289 | 11744 | 20023 | -4.1% | 0.90 | BELOW FLOOR |
| P vs M | commits_per_s | 5.09 | 0.15 | 5.11 | 0.05 | 0.11 | 0.59 | +0.4% | 0.18 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.04 | 43.8 | 0.26 | 0.18 | 0.07 | +0.8% | 1.87 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -713.8 | 504.4 | -664.5 | 410.1 | 459.7 | 2390 | -6.9% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 181.6 | 12.4 | 472.1 | 54.0 | 39.2 | 43.5 | +159.9% | 7.42 | **N better** |
| N vs H3 | q1_p99_us | 587.1 | 46.8 | 1734 | 411.9 | 293.1 | 1014 | +195.4% | 3.91 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 74786 | 7720 | 458.6 | 52.7 | 5459 | 5324 | -99.4% | 13.6 | **H3 better** |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 13967 | 5627 | 30229 | 10517 | 8434 | 12785 | +116.4% | 1.93 | no difference |
| N vs H3 | q6_p50_us | 54925 | 641.6 | 64645 | 4969 | 3543 | 7561 | +17.7% | 2.74 | no difference |
| N vs H3 | c2_q1_p50_us | 149.6 | 8.08 | 462.2 | 18.8 | 14.5 | 58.2 | +209.0% | 21.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 3645 | 1124 | 3036 | 714.7 | 941.9 | 1318 | -16.7% | 0.65 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 93787 | 14248 | 472.2 | 45.4 | 10075 | 4400 | -99.5% | 9.26 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 173.4 | 18.9 | 699.1 | 64.6 | 47.6 | 324.4 | +303.2% | 11.0 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 8292 | 1404 | 3642 | 1087 | 1255 | 3650 | -56.1% | 3.70 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 118889 | 6270 | 726.1 | 124.2 | 4434 | 11960 | -99.4% | 26.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 384623 | 28347 | 3377 | 835.7 | 20053 | 47750 | -99.1% | 19.0 | **H3 better** |
| N vs H3 | write_p50_us | 514.6 | 35.5 | 1428 | 124.3 | 91.4 | 60.5 | +177.6% | 10.00 | **N better** |
| N vs H3 | write_p99_us | 1016 | 218.6 | 4303 | 389.9 | 316.1 | 1554 | +323.4% | 10.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | commits_per_s | 1724 | 86.1 | 537.2 | 81.1 | 83.6 | 264.9 | -68.8% | 14.2 | **N better** |
| N vs H3 | pss_mib | 209.6 | 25.0 | 103.7 | 0.16 | 17.7 | 105.4 | -50.5% | 5.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_growth_bytes_per_key_read | 59104 | 4618 | 17964 | 3499 | 4097 | 40538 | -69.6% | 10.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 717.9 | 38.7 | 472.1 | 54.0 | 47.0 | 183.1 | -34.2% | 5.23 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2567 | 678.2 | 1734 | 411.9 | 561.1 | 1749 | -32.4% | 1.48 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 567.5 | 22.3 | 458.6 | 52.7 | 40.5 | 73.1 | -19.2% | 2.69 | no difference |
| P+ vs H3 | q3_p50_us | 340.0 | 51.7 | 437.4 | 45.8 | 48.8 | 209.5 | +28.7% | 1.99 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | 16398 | 1125 | 14162 | 352.4 | 833.4 | 3536 | -13.6% | 2.68 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 28528 | 4761 | 30229 | 10517 | 8163 | 13239 | +6.0% | 0.21 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 61881 | 3072 | 64645 | 4969 | 4131 | 8616 | +4.5% | 0.67 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 959.7 | 80.4 | 462.2 | 18.8 | 58.4 | 287.1 | -51.8% | 8.52 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q1_p99_us | 7614 | 1296 | 3036 | 714.7 | 1046 | 2788 | -60.1% | 4.37 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p50_us | 728.9 | 112.9 | 472.2 | 45.4 | 86.1 | 98.4 | -35.2% | 2.98 | no difference |
| P+ vs H3 | c4_q1_p50_us | 2005 | 102.2 | 699.1 | 64.6 | 85.5 | 321.1 | -65.1% | 15.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 10796 | 832.1 | 3642 | 1087 | 967.8 | 3532 | -66.3% | 7.39 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1834 | 144.4 | 726.1 | 124.2 | 134.7 | 682.2 | -60.4% | 8.23 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 10553 | 2249 | 3377 | 835.7 | 1697 | 1252 | -68.0% | 4.23 | **H3 better** |
| P+ vs H3 | write_p50_us | 1093 | 35.6 | 1428 | 124.3 | 91.4 | 44.4 | +30.6% | 3.66 | **P+ better** |
| P+ vs H3 | write_p99_us | 3808 | 471.6 | 4303 | 389.9 | 432.7 | 3296 | +13.0% | 1.14 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | commits_per_s | 750.1 | 51.3 | 537.2 | 81.1 | 67.8 | 184.6 | -28.4% | 3.14 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.06 | 103.7 | 0.16 | 0.12 | 0.11 | +139.2% | 497.6 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -757.9 | 518.4 | 17964 | 3499 | 2501 | 39876 | -2470.2% | 7.49 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 181.6 | 12.4 | 1452 | 92.6 | 66.0 | 159.0 | +699.3% | 19.2 | **N better** |
| N vs T | q1_p99_us | 587.1 | 46.8 | 4067 | 1135 | 803.1 | 2023 | +592.7% | 4.33 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | q2_p50_us | 74786 | 7720 | 1478 | 44.9 | 5459 | 5325 | -98.0% | 13.4 | **T better** |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 149.6 | 8.08 | 1985 | 106.7 | 75.6 | 60.6 | +1227.4% | 24.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q1_p99_us | 3645 | 1124 | 5145 | 1061 | 1093 | 1931 | +41.2% | 1.37 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c2_q2_p50_us | 93787 | 14248 | 2043 | 152.3 | 10075 | 4400 | -97.8% | 9.11 | **T better** |
| N vs T | c4_q1_p50_us | 173.4 | 18.9 | 2497 | 199.6 | 141.8 | 458.0 | +1340.2% | 16.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 8292 | 1404 | 8619 | 3114 | 2415 | 3748 | +3.9% | 0.14 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p50_us | 118889 | 6270 | 2603 | 291.6 | 4438 | 11950 | -97.8% | 26.2 | **T better** |
| N vs T | c4_q2_p99_us | 384623 | 28347 | 7038 | 2002 | 20094 | 47792 | -98.2% | 18.8 | **T better** |
| N vs T | write_p50_us | 514.6 | 35.5 | 1380 | 59.0 | 48.7 | 219.8 | +168.2% | 17.8 | **N better** |
| N vs T | write_p99_us | 1016 | 218.6 | 3074 | 1342 | 961.1 | 970.1 | +202.5% | 2.14 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | commits_per_s | 1724 | 86.1 | 686.2 | 43.6 | 68.2 | 252.7 | -60.2% | 15.2 | **N better** |
| N vs T | pss_mib | 209.6 | 25.0 | 2584 | 0.12 | 17.7 | 105.4 | +1132.8% | 134.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | pss_growth_bytes_per_key_read | 59104 | 4618 | -48.1 | 44.8 | 3266 | 7521 | -100.1% | 18.1 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | q1_p50_us | 717.9 | 38.7 | 1452 | 92.6 | 71.0 | 238.6 | +102.2% | 10.3 | **P+ better** |
| P+ vs T | q1_p99_us | 2567 | 678.2 | 4067 | 1135 | 934.8 | 2474 | +58.5% | 1.60 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | q2_p50_us | 567.5 | 22.3 | 1478 | 44.9 | 35.5 | 149.9 | +160.5% | 25.7 | **P+ better** |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 959.7 | 80.4 | 1985 | 106.7 | 94.5 | 287.6 | +106.9% | 10.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c2_q1_p99_us | 7614 | 1296 | 5145 | 1061 | 1184 | 3125 | -32.4% | 2.08 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c2_q2_p50_us | 728.9 | 112.9 | 2043 | 152.3 | 134.1 | 96.4 | +180.3% | 9.80 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2005 | 102.2 | 2497 | 199.6 | 158.6 | 455.7 | +24.5% | 3.10 | **P+ better** |
| P+ vs T | c4_q1_p99_us | 10796 | 832.1 | 8619 | 3114 | 2279 | 3634 | -20.2% | 0.96 | BELOW FLOOR |
| P+ vs T | c4_q2_p50_us | 1834 | 144.4 | 2603 | 291.6 | 230.1 | 464.8 | +41.9% | 3.34 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 10553 | 2249 | 7038 | 2002 | 2129 | 2359 | -33.3% | 1.65 | no difference |
| P+ vs T | write_p50_us | 1093 | 35.6 | 1380 | 59.0 | 48.7 | 215.9 | +26.2% | 5.88 | **P+ better** |
| P+ vs T | write_p99_us | 3808 | 471.6 | 3074 | 1342 | 1006 | 3064 | -19.3% | 0.73 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | commits_per_s | 750.1 | 51.3 | 686.2 | 43.6 | 47.6 | 166.7 | -8.5% | 1.34 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.06 | 2584 | 0.12 | 0.09 | 0.22 | +5863.0% | 27241 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -757.9 | 518.4 | -48.1 | 44.8 | 367.9 | 1829 | -93.7% | 1.93 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 472.1 | 54.0 | 1452 | 92.6 | 75.8 | 163.4 | +207.5% | 12.9 | **H3 better** |
| H3 vs T | q1_p99_us | 1734 | 411.9 | 4067 | 1135 | 853.7 | 2263 | +134.5% | 2.73 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | q2_p50_us | 458.6 | 52.7 | 1478 | 44.9 | 49.0 | 132.3 | +222.3% | 20.8 | **H3 better** |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 462.2 | 18.8 | 1985 | 106.7 | 76.6 | 44.2 | +329.6% | 19.9 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 3036 | 714.7 | 5145 | 1061 | 904.8 | 1414 | +69.4% | 2.33 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c2_q2_p50_us | 472.2 | 45.4 | 2043 | 152.3 | 112.4 | 39.5 | +332.7% | 14.0 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 699.1 | 64.6 | 2497 | 199.6 | 148.4 | 551.3 | +257.2% | 12.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 3642 | 1087 | 8619 | 3114 | 2332 | 1844 | +136.6% | 2.13 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p50_us | 726.1 | 124.2 | 2603 | 291.6 | 224.1 | 511.5 | +258.4% | 8.37 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p99_us | 3377 | 835.7 | 7038 | 2002 | 1534 | 2136 | +108.4% | 2.39 | no difference |
| H3 vs T | write_p50_us | 1428 | 124.3 | 1380 | 59.0 | 97.3 | 220.2 | -3.4% | 0.50 | BELOW FLOOR |
| H3 vs T | write_p99_us | 4303 | 389.9 | 3074 | 1342 | 987.9 | 1816 | -28.6% | 1.24 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | commits_per_s | 537.2 | 81.1 | 686.2 | 43.6 | 65.1 | 218.0 | +27.7% | 2.29 | BELOW FLOOR |
| H3 vs T | pss_mib | 103.7 | 0.16 | 2584 | 0.12 | 0.14 | 0.22 | +2392.6% | 17554 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 17964 | 3499 | -48.1 | 44.8 | 2474 | 39836 | -100.3% | 7.28 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 181.6 | 12.4 | 1040 | 107.9 | 76.8 | 54.4 | +472.8% | 11.2 | **N better** |
| N vs H1 | q1_p99_us | 587.1 | 46.8 | 4602 | 1234 | 873.4 | 1399 | +683.7% | 4.60 | **N better** |
| N vs H1 | q2_p50_us | 74786 | 7720 | 844.2 | 81.3 | 5459 | 5324 | -98.9% | 13.5 | **H1 better** |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 13967 | 5627 | 39716 | 2730 | 4423 | 9638 | +184.3% | 5.82 | **N better** |
| N vs H1 | q6_p50_us | 54925 | 641.6 | 11010 | 740.0 | 692.5 | 146.3 | -80.0% | 63.4 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 149.6 | 8.08 | 893.9 | 62.0 | 44.2 | 111.5 | +497.6% | 16.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q1_p99_us | 3645 | 1124 | 6370 | 746.6 | 954.2 | 1466 | +74.8% | 2.86 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q2_p50_us | 93787 | 14248 | 870.3 | 83.4 | 10075 | 4404 | -99.1% | 9.22 | **H1 better** |
| N vs H1 | c4_q1_p50_us | 173.4 | 18.9 | 960.3 | 32.0 | 26.3 | 74.7 | +453.8% | 29.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 8292 | 1404 | 5805 | 936.4 | 1193 | 3490 | -30.0% | 2.08 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p50_us | 118889 | 6270 | 945.8 | 47.8 | 4433 | 11950 | -99.2% | 26.6 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 384623 | 28347 | 4723 | 1154 | 20061 | 47747 | -98.8% | 18.9 | **H1 better** |
| N vs H1 | write_p50_us | 514.6 | 35.5 | 1735 | 133.2 | 97.5 | 83.8 | +237.2% | 12.5 | **N better** |
| N vs H1 | write_p99_us | 1016 | 218.6 | 6332 | 394.0 | 318.6 | 278.1 | +523.0% | 16.7 | **N better** |
| N vs H1 | commits_per_s | 1724 | 86.1 | 457.4 | 41.3 | 67.5 | 208.6 | -73.5% | 18.8 | **N better** |
| N vs H1 | pss_mib | 209.6 | 25.0 | 414.5 | 17.5 | 21.6 | 106.5 | +97.8% | 9.49 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | pss_growth_bytes_per_key_read | 59104 | 4618 | 14357 | 1993 | 3557 | 22701 | -75.7% | 12.6 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q1_p50_us | 717.9 | 38.7 | 1040 | 107.9 | 81.1 | 186.0 | +44.9% | 3.98 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2567 | 678.2 | 4602 | 1234 | 995.8 | 1997 | +79.3% | 2.04 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q2_p50_us | 567.5 | 22.3 | 844.2 | 81.3 | 59.6 | 72.1 | +48.8% | 4.64 | **P+ better** |
| P+ vs H1 | q3_p50_us | 340.0 | 51.7 | 1004 | 107.1 | 84.1 | 227.5 | +195.4% | 7.90 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q4_p50_us | 16398 | 1125 | 30237 | 1122 | 1124 | 3396 | +84.4% | 12.3 | **P+ better** |
| P+ vs H1 | q5_p50_us | 28528 | 4761 | 39716 | 2730 | 3881 | 10233 | +39.2% | 2.88 | no difference |
| P+ vs H1 | q6_p50_us | 61881 | 3072 | 11010 | 740.0 | 2234 | 4135 | -82.2% | 22.8 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 959.7 | 80.4 | 893.9 | 62.0 | 71.8 | 302.5 | -6.9% | 0.92 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c2_q1_p99_us | 7614 | 1296 | 6370 | 746.6 | 1058 | 2861 | -16.3% | 1.18 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c2_q2_p50_us | 728.9 | 112.9 | 870.3 | 83.4 | 99.3 | 208.9 | +19.4% | 1.42 | BELOW FLOOR |
| P+ vs H1 | c4_q1_p50_us | 2005 | 102.2 | 960.3 | 32.0 | 75.7 | 58.8 | -52.1% | 13.8 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 10796 | 832.1 | 5805 | 936.4 | 885.8 | 3367 | -46.2% | 5.63 | **H1 better** |
| P+ vs H1 | c4_q2_p50_us | 1834 | 144.4 | 945.8 | 47.8 | 107.5 | 462.6 | -48.4% | 8.26 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 10553 | 2249 | 4723 | 1154 | 1788 | 1139 | -55.2% | 3.26 | **H1 better** |
| P+ vs H1 | write_p50_us | 1093 | 35.6 | 1735 | 133.2 | 97.5 | 73.0 | +58.7% | 6.58 | **P+ better** |
| P+ vs H1 | write_p99_us | 3808 | 471.6 | 6332 | 394.0 | 434.5 | 2920 | +66.3% | 5.81 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | commits_per_s | 750.1 | 51.3 | 457.4 | 41.3 | 46.5 | 86.3 | -39.0% | 6.29 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.06 | 414.5 | 17.5 | 12.4 | 15.2 | +856.6% | 30.0 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -757.9 | 518.4 | 14357 | 1993 | 1456 | 21497 | -1994.2% | 10.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 181.6 | 12.4 | 129.3 | 13.1 | 12.7 | 40.4 | -28.8% | 4.11 | **H2 better** |
| N vs H2 | q1_p99_us | 587.1 | 46.8 | 627.9 | 163.2 | 120.1 | 422.5 | +6.9% | 0.34 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 74786 | 7720 | 230.8 | 31.3 | 5459 | 5324 | -99.7% | 13.7 | **H2 better** |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 13967 | 5627 | 178.1 | 16.3 | 3979 | 204.6 | -98.7% | 3.47 | **H2 better** |
| N vs H2 | q6_p50_us | 54925 | 641.6 | 5251 | 1091 | 895.3 | 1126 | -90.4% | 55.5 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 149.6 | 8.08 | 113.6 | 20.5 | 15.6 | 56.4 | -24.1% | 2.31 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q1_p99_us | 3645 | 1124 | 1933 | 710.3 | 940.2 | 1505 | -47.0% | 1.82 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 93787 | 14248 | 225.9 | 11.1 | 10075 | 4401 | -99.8% | 9.29 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 173.4 | 18.9 | 203.2 | 36.9 | 29.3 | 77.4 | +17.2% | 1.02 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p99_us | 8292 | 1404 | 3279 | 415.6 | 1035 | 3981 | -60.5% | 4.84 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q2_p50_us | 118889 | 6270 | 380.8 | 63.2 | 4434 | 11950 | -99.7% | 26.7 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 384623 | 28347 | 4794 | 1048 | 20058 | 47747 | -98.8% | 18.9 | **H2 better** |
| N vs H2 | write_p50_us | 514.6 | 35.5 | 1550 | 57.1 | 47.6 | 177.5 | +201.3% | 21.8 | **N better** |
| N vs H2 | write_p99_us | 1016 | 218.6 | 6313 | 987.5 | 715.2 | 403.0 | +521.1% | 7.41 | **N better** |
| N vs H2 | commits_per_s | 1724 | 86.1 | 519.1 | 39.9 | 67.1 | 215.8 | -69.9% | 18.0 | **N better** |
| N vs H2 | pss_mib | 209.6 | 25.0 | 43.4 | 0.02 | 17.7 | 105.4 | -79.3% | 9.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_growth_bytes_per_key_read | 59104 | 4618 | -783.8 | 345.6 | 3275 | 7737 | -101.3% | 18.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p50_us | 717.9 | 38.7 | 129.3 | 13.1 | 28.9 | 182.4 | -82.0% | 20.4 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2567 | 678.2 | 627.9 | 163.2 | 493.2 | 1486 | -75.5% | 3.93 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q2_p50_us | 567.5 | 22.3 | 230.8 | 31.3 | 27.2 | 90.1 | -59.3% | 12.4 | **H2 better** |
| P+ vs H2 | q3_p50_us | 340.0 | 51.7 | 294.1 | 44.0 | 48.0 | 301.2 | -13.5% | 0.96 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q4_p50_us | 16398 | 1125 | 1158 | 42.0 | 795.9 | 362.4 | -92.9% | 19.1 | **H2 better** |
| P+ vs H2 | q5_p50_us | 28528 | 4761 | 178.1 | 16.3 | 3366 | 3444 | -99.4% | 8.42 | **H2 better** |
| P+ vs H2 | q6_p50_us | 61881 | 3072 | 5251 | 1091 | 2305 | 4283 | -91.5% | 24.6 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 959.7 | 80.4 | 113.6 | 20.5 | 58.7 | 286.8 | -88.2% | 14.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q1_p99_us | 7614 | 1296 | 1933 | 710.3 | 1045 | 2881 | -74.6% | 5.44 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 728.9 | 112.9 | 225.9 | 11.1 | 80.2 | 110.8 | -69.0% | 6.27 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 2005 | 102.2 | 203.2 | 36.9 | 76.8 | 62.1 | -89.9% | 23.5 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 10796 | 832.1 | 3279 | 415.6 | 657.7 | 3874 | -69.6% | 11.4 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p50_us | 1834 | 144.4 | 380.8 | 63.2 | 111.4 | 464.1 | -79.2% | 13.0 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 10553 | 2249 | 4794 | 1048 | 1755 | 1135 | -54.6% | 3.28 | **H2 better** |
| P+ vs H2 | write_p50_us | 1093 | 35.6 | 1550 | 57.1 | 47.6 | 172.7 | +41.8% | 9.60 | **P+ better** |
| P+ vs H2 | write_p99_us | 3808 | 471.6 | 6313 | 987.5 | 773.8 | 2935 | +65.8% | 3.24 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 750.1 | 51.3 | 519.1 | 39.9 | 45.9 | 102.6 | -30.8% | 5.03 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.06 | 43.4 | 0.02 | 0.04 | 0.09 | +0.1% | 0.70 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | -757.9 | 518.4 | -783.8 | 345.6 | 440.6 | 2578 | +3.4% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 181.9 | 14.1 | 698.4 | 23.6 | 19.4 | 40.9 | +284.0% | 26.6 | **N better** |
| N vs P+ | q1_p99_us | 826.1 | 269.5 | 2522 | 725.7 | 547.4 | 2611 | +205.3% | 3.10 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 79056 | 9339 | 547.5 | 37.3 | 6603 | 22143 | -99.3% | 11.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 9918 | 846.8 | 19875 | 4471 | 3218 | 23791 | +100.4% | 3.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q6_p50_us | 55757 | 1827 | 59348 | 5453 | 4067 | 5963 | +6.4% | 0.88 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 170.4 | 17.5 | 863.4 | 66.8 | 48.8 | 112.7 | +406.8% | 14.2 | **N better** |
| N vs P+ | c2_q1_p99_us | 2037 | 1410 | 6369 | 1153 | 1288 | 1451 | +212.7% | 3.36 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 83146 | 9654 | 718.8 | 54.0 | 6826 | 904.3 | -99.1% | 12.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 181.9 | 10.8 | 1986 | 156.1 | 110.6 | 63.5 | +991.8% | 16.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 7949 | 1067 | 9694 | 1024 | 1046 | 6284 | +22.0% | 1.67 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 116656 | 9644 | 1649 | 53.0 | 6820 | 8698 | -98.6% | 16.9 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 412461 | 18514 | 8515 | 2758 | 13236 | 40876 | -97.9% | 30.5 | **P+ better** |
| N vs P+ | write_p50_us | 533.2 | 18.5 | 1031 | 44.5 | 34.1 | 101.5 | +93.3% | 14.6 | **N better** |
| N vs P+ | write_p99_us | 1997 | 1168 | 3279 | 158.6 | 833.4 | 1117 | +64.2% | 1.54 | no difference |
| N vs P+ | commits_per_s | 1411 | 240.1 | 831.2 | 30.9 | 171.2 | 194.6 | -41.1% | 3.39 | **N better** |
| N vs P+ | pss_mib | 223.5 | 0.80 | 43.4 | 0.08 | 0.57 | 96.0 | -80.6% | 317.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_growth_bytes_per_key_read | 70572 | 7647 | -749.1 | 533.3 | 5420 | 17747 | -101.1% | 13.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 181.9 | 14.1 | 111.9 | 12.5 | 13.3 | 11.7 | -38.5% | 5.24 | **P better** |
| N vs P | q1_p99_us | 826.1 | 269.5 | 654.5 | 142.3 | 215.5 | 367.4 | -20.8% | 0.80 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 79056 | 9339 | 193.9 | 27.7 | 6603 | 22143 | -99.8% | 11.9 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 9918 | 846.8 | 149.0 | 40.2 | 599.4 | 174.1 | -98.5% | 16.3 | **P better** |
| N vs P | q6_p50_us | 55757 | 1827 | 5140 | 797.9 | 1410 | 1486 | -90.8% | 35.9 | **P better** |
| N vs P | c2_q1_p50_us | 170.4 | 17.5 | 122.2 | 49.5 | 37.2 | 12.0 | -28.2% | 1.29 | no difference |
| N vs P | c2_q1_p99_us | 2037 | 1410 | 2310 | 340.4 | 1025 | 2518 | +13.4% | 0.27 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 83146 | 9654 | 245.9 | 33.2 | 6826 | 850.4 | -99.7% | 12.1 | **P better** |
| N vs P | c4_q1_p50_us | 181.9 | 10.8 | 189.9 | 47.9 | 34.7 | 45.4 | +4.4% | 0.23 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 7949 | 1067 | 2767 | 424.7 | 812.3 | 848.2 | -65.2% | 6.38 | **P better** |
| N vs P | c4_q2_p50_us | 116656 | 9644 | 371.5 | 34.6 | 6820 | 8697 | -99.7% | 17.1 | **P better** |
| N vs P | c4_q2_p99_us | 412461 | 18514 | 5394 | 1588 | 13140 | 40708 | -98.7% | 31.0 | **P better** |
| N vs P | write_p50_us | 533.2 | 18.5 | 193126 | 3857 | 2727 | 1931 | +36119.2% | 70.6 | **N better** |
| N vs P | write_p99_us | 1997 | 1168 | 246322 | 5616 | 4056 | 20007 | +12235.4% | 60.2 | **N better** |
| N vs P | commits_per_s | 1411 | 240.1 | 5.03 | 0.06 | 169.8 | 178.2 | -99.6% | 8.28 | **N better** |
| N vs P | pss_mib | 223.5 | 0.80 | 43.5 | 0.08 | 0.57 | 96.0 | -80.5% | 317.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_growth_bytes_per_key_read | 70572 | 7647 | -717.6 | 476.5 | 5418 | 17655 | -101.0% | 13.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 698.4 | 23.6 | 111.9 | 12.5 | 18.9 | 39.2 | -84.0% | 31.1 | **P better** |
| P+ vs P | q1_p99_us | 2522 | 725.7 | 654.5 | 142.3 | 522.9 | 2634 | -74.1% | 3.57 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 547.5 | 37.3 | 193.9 | 27.7 | 32.9 | 93.1 | -64.6% | 10.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 342.2 | 20.5 | 231.9 | 44.9 | 34.9 | 47.2 | -32.2% | 3.16 | **P better** |
| P+ vs P | q4_p50_us | 15981 | 734.4 | 1122 | 91.5 | 523.3 | 830.2 | -93.0% | 28.4 | **P better** |
| P+ vs P | q5_p50_us | 19875 | 4471 | 149.0 | 40.2 | 3162 | 23790 | -99.3% | 6.24 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q6_p50_us | 59348 | 5453 | 5140 | 797.9 | 3897 | 5981 | -91.3% | 13.9 | **P better** |
| P+ vs P | c2_q1_p50_us | 863.4 | 66.8 | 122.2 | 49.5 | 58.8 | 113.3 | -85.8% | 12.6 | **P better** |
| P+ vs P | c2_q1_p99_us | 6369 | 1153 | 2310 | 340.4 | 850.4 | 2062 | -63.7% | 4.77 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 718.8 | 54.0 | 245.9 | 33.2 | 44.8 | 307.6 | -65.8% | 10.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p50_us | 1986 | 156.1 | 189.9 | 47.9 | 115.5 | 59.8 | -90.4% | 15.6 | **P better** |
| P+ vs P | c4_q1_p99_us | 9694 | 1024 | 2767 | 424.7 | 784.1 | 6283 | -71.5% | 8.83 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1649 | 53.0 | 371.5 | 34.6 | 44.8 | 177.7 | -77.5% | 28.5 | **P better** |
| P+ vs P | c4_q2_p99_us | 8515 | 2758 | 5394 | 1588 | 2251 | 3718 | -36.7% | 1.39 | BELOW FLOOR |
| P+ vs P | write_p50_us | 1031 | 44.5 | 193126 | 3857 | 2727 | 1933 | +18640.6% | 70.4 | **P+ better** |
| P+ vs P | write_p99_us | 3279 | 158.6 | 246322 | 5616 | 3973 | 20029 | +7412.3% | 61.2 | **P+ better** |
| P+ vs P | commits_per_s | 831.2 | 30.9 | 5.03 | 0.06 | 21.8 | 78.3 | -99.4% | 37.8 | **P+ better** |
| P+ vs P | pss_mib | 43.4 | 0.08 | 43.5 | 0.08 | 0.08 | 0.25 | +0.4% | 2.41 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -749.1 | 533.3 | -717.6 | 476.5 | 505.7 | 2051 | -4.2% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 111.9 | 12.5 | 96.8 | 22.4 | 18.1 | 10.0 | -13.5% | 0.84 | no difference |
| P vs M | q1_p99_us | 654.5 | 142.3 | 800.8 | 344.0 | 263.2 | 377.9 | +22.4% | 0.56 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 193.9 | 27.7 | 189.9 | 31.4 | 29.6 | 105.1 | -2.1% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 231.9 | 44.9 | 241.3 | 29.3 | 37.9 | 53.6 | +4.1% | 0.25 | BELOW FLOOR |
| P vs M | q4_p50_us | 1122 | 91.5 | 1077 | 98.7 | 95.2 | 142.8 | -4.0% | 0.47 | BELOW FLOOR |
| P vs M | q5_p50_us | 149.0 | 40.2 | 149.9 | 30.2 | 35.6 | 30.1 | +0.6% | 0.03 | BELOW FLOOR |
| P vs M | q6_p50_us | 5140 | 797.9 | 5968 | 223.7 | 586.0 | 1209 | +16.1% | 1.41 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 122.2 | 49.5 | 133.9 | 43.2 | 46.5 | 14.1 | +9.5% | 0.25 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 2310 | 340.4 | 1862 | 411.3 | 377.5 | 2154 | -19.4% | 1.19 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 245.9 | 33.2 | 262.4 | 28.1 | 30.8 | 18.4 | +6.7% | 0.54 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 189.9 | 47.9 | 211.5 | 54.3 | 51.2 | 28.4 | +11.3% | 0.42 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2767 | 424.7 | 3238 | 466.2 | 445.9 | 666.5 | +17.0% | 1.06 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 371.5 | 34.6 | 432.6 | 55.0 | 46.0 | 106.0 | +16.4% | 1.33 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 5394 | 1588 | 3516 | 626.6 | 1207 | 208.9 | -34.8% | 1.56 | no difference |
| P vs M | write_p50_us | 193126 | 3857 | 189513 | 2541 | 3266 | 10641 | -1.9% | 1.11 | BELOW FLOOR |
| P vs M | write_p99_us | 246322 | 5616 | 243919 | 9230 | 7640 | 55054 | -1.0% | 0.31 | BELOW FLOOR |
| P vs M | commits_per_s | 5.03 | 0.06 | 5.11 | 0.12 | 0.09 | 0.20 | +1.5% | 0.78 | BELOW FLOOR |
| P vs M | pss_mib | 43.5 | 0.08 | 43.4 | 0.04 | 0.06 | 0.22 | -0.2% | 1.69 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -717.6 | 476.5 | -736.9 | 444.5 | 460.8 | 2060 | +2.7% | 0.04 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 181.9 | 14.1 | 438.5 | 9.37 | 12.0 | 89.3 | +141.1% | 21.4 | **N better** |
| N vs H3 | q1_p99_us | 826.1 | 269.5 | 1510 | 117.1 | 207.8 | 1365 | +82.7% | 3.29 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 79056 | 9339 | 447.7 | 48.1 | 6603 | 22144 | -99.4% | 11.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 9918 | 846.8 | 23858 | 5897 | 4213 | 43934 | +140.6% | 3.31 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q6_p50_us | 55757 | 1827 | 68322 | 5399 | 4030 | 1905 | +22.5% | 3.12 | **N better** |
| N vs H3 | c2_q1_p50_us | 170.4 | 17.5 | 430.5 | 67.1 | 49.0 | 9.28 | +152.7% | 5.31 | **N better** |
| N vs H3 | c2_q1_p99_us | 2037 | 1410 | 2342 | 614.0 | 1087 | 2552 | +15.0% | 0.28 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 83146 | 9654 | 406.0 | 48.7 | 6826 | 859.3 | -99.5% | 12.1 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 181.9 | 10.8 | 760.2 | 124.1 | 88.1 | 188.7 | +317.8% | 6.57 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 7949 | 1067 | 3076 | 719.8 | 910.3 | 1277 | -61.3% | 5.35 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 116656 | 9644 | 720.2 | 98.4 | 6820 | 8697 | -99.4% | 17.0 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 412461 | 18514 | 2982 | 754.3 | 13102 | 40860 | -99.3% | 31.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 533.2 | 18.5 | 1489 | 58.5 | 43.4 | 59.2 | +179.3% | 22.0 | **N better** |
| N vs H3 | write_p99_us | 1997 | 1168 | 4977 | 872.1 | 1031 | 1038 | +149.3% | 2.89 | no difference |
| N vs H3 | commits_per_s | 1411 | 240.1 | 562.7 | 39.6 | 172.1 | 235.1 | -60.1% | 4.93 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | pss_mib | 223.5 | 0.80 | 113.3 | 1.98 | 1.51 | 96.0 | -49.3% | 72.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_growth_bytes_per_key_read | 70572 | 7647 | 21334 | 3734 | 6017 | 45330 | -69.8% | 8.18 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 698.4 | 23.6 | 438.5 | 9.37 | 17.9 | 96.8 | -37.2% | 14.5 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2522 | 725.7 | 1510 | 117.1 | 519.8 | 2944 | -40.2% | 1.95 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 547.5 | 37.3 | 447.7 | 48.1 | 43.1 | 116.5 | -18.2% | 2.32 | BELOW FLOOR |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 342.2 | 20.5 | 429.0 | 45.5 | 35.3 | 125.2 | +25.4% | 2.46 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 15981 | 734.4 | 15079 | 700.0 | 717.4 | 2365 | -5.6% | 1.26 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 19875 | 4471 | 23858 | 5897 | 5233 | 49962 | +20.0% | 0.76 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q6_p50_us | 59348 | 5453 | 68322 | 5399 | 5426 | 6098 | +15.1% | 1.65 | no difference |
| P+ vs H3 | c2_q1_p50_us | 863.4 | 66.8 | 430.5 | 67.1 | 66.9 | 113.1 | -50.1% | 6.47 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 6369 | 1153 | 2342 | 614.0 | 924.0 | 2104 | -63.2% | 4.36 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 718.8 | 54.0 | 406.0 | 48.7 | 51.4 | 331.5 | -43.5% | 6.08 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 1986 | 156.1 | 760.2 | 124.1 | 141.0 | 192.7 | -61.7% | 8.70 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 9694 | 1024 | 3076 | 719.8 | 885.2 | 6355 | -68.3% | 7.48 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1649 | 53.0 | 720.2 | 98.4 | 79.0 | 176.8 | -56.3% | 11.7 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8515 | 2758 | 2982 | 754.3 | 2022 | 5121 | -65.0% | 2.74 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1031 | 44.5 | 1489 | 58.5 | 52.0 | 115.1 | +44.5% | 8.82 | **P+ better** |
| P+ vs H3 | write_p99_us | 3279 | 158.6 | 4977 | 872.1 | 626.8 | 1390 | +51.8% | 2.71 | no difference |
| P+ vs H3 | commits_per_s | 831.2 | 30.9 | 562.7 | 39.6 | 35.5 | 172.1 | -32.3% | 7.56 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | pss_mib | 43.4 | 0.08 | 113.3 | 1.98 | 1.40 | 0.53 | +161.3% | 49.8 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -749.1 | 533.3 | 21334 | 3734 | 2667 | 41801 | -2947.9% | 8.28 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 181.9 | 14.1 | 1437 | 44.6 | 33.1 | 172.1 | +690.0% | 37.9 | **N better** |
| N vs T | q1_p99_us | 826.1 | 269.5 | 8059 | 2494 | 1774 | 278.8 | +875.6% | 4.08 | **N better** |
| N vs T | q2_p50_us | 79056 | 9339 | 1429 | 100.7 | 6604 | 22146 | -98.2% | 11.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 170.4 | 17.5 | 1979 | 84.1 | 60.7 | 27.7 | +1061.8% | 29.8 | **N better** |
| N vs T | c2_q1_p99_us | 2037 | 1410 | 5108 | 1489 | 1450 | 1649 | +150.8% | 2.12 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q2_p50_us | 83146 | 9654 | 1989 | 75.7 | 6827 | 852.1 | -97.6% | 11.9 | **T better** |
| N vs T | c4_q1_p50_us | 181.9 | 10.8 | 2453 | 118.8 | 84.3 | 111.7 | +1248.3% | 26.9 | **N better** |
| N vs T | c4_q1_p99_us | 7949 | 1067 | 7013 | 1787 | 1472 | 1801 | -11.8% | 0.64 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c4_q2_p50_us | 116656 | 9644 | 2476 | 185.6 | 6821 | 8697 | -97.9% | 16.7 | **T better** |
| N vs T | c4_q2_p99_us | 412461 | 18514 | 5848 | 795.4 | 13104 | 40711 | -98.6% | 31.0 | **T better** |
| N vs T | write_p50_us | 533.2 | 18.5 | 1366 | 29.8 | 24.8 | 472.5 | +156.1% | 33.6 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p99_us | 1997 | 1168 | 8115 | 878.2 | 1033 | 1026 | +306.4% | 5.92 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | commits_per_s | 1411 | 240.1 | 587.4 | 61.6 | 175.3 | 180.6 | -58.4% | 4.70 | **N better** |
| N vs T | pss_mib | 223.5 | 0.80 | 2581 | 0.18 | 0.58 | 96.0 | +1054.8% | 4068 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | pss_growth_bytes_per_key_read | 70572 | 7647 | -170.4 | 92.9 | 5408 | 17660 | -100.2% | 13.1 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | q1_p50_us | 698.4 | 23.6 | 1437 | 44.6 | 35.7 | 176.1 | +105.7% | 20.7 | **P+ better** |
| P+ vs T | q1_p99_us | 2522 | 725.7 | 8059 | 2494 | 1837 | 2623 | +219.5% | 3.01 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | q2_p50_us | 547.5 | 37.3 | 1429 | 100.7 | 75.9 | 329.4 | +161.1% | 11.6 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 863.4 | 66.8 | 1979 | 84.1 | 75.9 | 116.0 | +129.2% | 14.7 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 6369 | 1153 | 5108 | 1489 | 1332 | 796.4 | -19.8% | 0.95 | no difference |
| P+ vs T | c2_q2_p50_us | 718.8 | 54.0 | 1989 | 75.7 | 65.8 | 312.3 | +176.7% | 19.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q1_p50_us | 1986 | 156.1 | 2453 | 118.8 | 138.7 | 118.3 | +23.5% | 3.36 | **P+ better** |
| P+ vs T | c4_q1_p99_us | 9694 | 1024 | 7013 | 1787 | 1457 | 6481 | -27.7% | 1.84 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c4_q2_p50_us | 1649 | 53.0 | 2476 | 185.6 | 136.5 | 165.4 | +50.2% | 6.06 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 8515 | 2758 | 5848 | 795.4 | 2030 | 3755 | -31.3% | 1.31 | BELOW FLOOR |
| P+ vs T | write_p50_us | 1031 | 44.5 | 1366 | 29.8 | 37.9 | 482.7 | +32.5% | 8.85 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | write_p99_us | 3279 | 158.6 | 8115 | 878.2 | 631.0 | 1381 | +147.5% | 7.66 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 831.2 | 30.9 | 587.4 | 61.6 | 48.7 | 83.6 | -29.3% | 5.00 | **P+ better** |
| P+ vs T | pss_mib | 43.4 | 0.08 | 2581 | 0.18 | 0.14 | 0.30 | +5853.7% | 18322 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -749.1 | 533.3 | -170.4 | 92.9 | 382.8 | 2091 | -77.2% | 1.51 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 438.5 | 9.37 | 1437 | 44.6 | 32.2 | 193.2 | +227.7% | 31.0 | **H3 better** |
| H3 vs T | q1_p99_us | 1510 | 117.1 | 8059 | 2494 | 1766 | 1388 | +433.9% | 3.71 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q2_p50_us | 447.7 | 48.1 | 1429 | 100.7 | 78.9 | 348.4 | +219.3% | 12.4 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 430.5 | 67.1 | 1979 | 84.1 | 76.1 | 29.2 | +359.8% | 20.4 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 2342 | 614.0 | 5108 | 1489 | 1139 | 2245 | +118.1% | 2.43 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q2_p50_us | 406.0 | 48.7 | 1989 | 75.7 | 63.7 | 135.1 | +390.0% | 24.9 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 760.2 | 124.1 | 2453 | 118.8 | 121.5 | 213.5 | +222.7% | 13.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 3076 | 719.8 | 7013 | 1787 | 1363 | 2035 | +127.9% | 2.89 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c4_q2_p50_us | 720.2 | 98.4 | 2476 | 185.6 | 148.6 | 62.3 | +243.9% | 11.8 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 2982 | 754.3 | 5848 | 795.4 | 775.1 | 3572 | +96.1% | 3.70 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | write_p50_us | 1489 | 58.5 | 1366 | 29.8 | 46.4 | 475.6 | -8.3% | 2.66 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | write_p99_us | 4977 | 872.1 | 8115 | 878.2 | 875.2 | 1318 | +63.0% | 3.58 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 562.7 | 39.6 | 587.4 | 61.6 | 51.8 | 156.1 | +4.4% | 0.48 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | pss_mib | 113.3 | 1.98 | 2581 | 0.18 | 1.41 | 0.52 | +2178.2% | 1751 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 21334 | 3734 | -170.4 | 92.9 | 2641 | 41764 | -100.8% | 8.14 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 181.9 | 14.1 | 1001 | 34.8 | 26.5 | 188.8 | +450.5% | 30.9 | **N better** |
| N vs H1 | q1_p99_us | 826.1 | 269.5 | 5265 | 689.9 | 523.7 | 3445 | +537.3% | 8.48 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q2_p50_us | 79056 | 9339 | 847.0 | 54.4 | 6604 | 22144 | -98.9% | 11.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 9918 | 846.8 | 42516 | 41608 | 29428 | 3869 | +328.7% | 1.11 | no difference |
| N vs H1 | q6_p50_us | 55757 | 1827 | 10199 | 1385 | 1621 | 999.8 | -81.7% | 28.1 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 170.4 | 17.5 | 953.8 | 54.5 | 40.5 | 39.0 | +459.9% | 19.4 | **N better** |
| N vs H1 | c2_q1_p99_us | 2037 | 1410 | 6307 | 1588 | 1501 | 1548 | +209.7% | 2.84 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q2_p50_us | 83146 | 9654 | 933.1 | 45.8 | 6826 | 860.9 | -98.9% | 12.0 | **H1 better** |
| N vs H1 | c4_q1_p50_us | 181.9 | 10.8 | 1058 | 88.1 | 62.8 | 48.7 | +481.5% | 14.0 | **N better** |
| N vs H1 | c4_q1_p99_us | 7949 | 1067 | 5326 | 506.7 | 835.4 | 914.7 | -33.0% | 3.14 | **H1 better** |
| N vs H1 | c4_q2_p50_us | 116656 | 9644 | 999.8 | 39.2 | 6820 | 8698 | -99.1% | 17.0 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 412461 | 18514 | 6565 | 2583 | 13218 | 40716 | -98.4% | 30.7 | **H1 better** |
| N vs H1 | write_p50_us | 533.2 | 18.5 | 1751 | 160.3 | 114.1 | 17.3 | +228.3% | 10.7 | **N better** |
| N vs H1 | write_p99_us | 1997 | 1168 | 4985 | 591.9 | 925.8 | 1168 | +149.6% | 3.23 | **N better** |
| N vs H1 | commits_per_s | 1411 | 240.1 | 493.0 | 26.8 | 170.8 | 193.2 | -65.1% | 5.37 | **N better** |
| N vs H1 | pss_mib | 223.5 | 0.80 | 419.9 | 5.70 | 4.07 | 102.6 | +87.9% | 48.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | pss_growth_bytes_per_key_read | 70572 | 7647 | 19137 | 3632 | 5986 | 30741 | -72.9% | 8.59 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q1_p50_us | 698.4 | 23.6 | 1001 | 34.8 | 29.7 | 192.5 | +43.4% | 10.2 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2522 | 725.7 | 5265 | 689.9 | 708.0 | 4321 | +108.7% | 3.87 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | q2_p50_us | 547.5 | 37.3 | 847.0 | 54.4 | 46.6 | 182.7 | +54.7% | 6.42 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 342.2 | 20.5 | 913.7 | 63.9 | 47.5 | 252.2 | +167.0% | 12.0 | **P+ better** |
| P+ vs H1 | q4_p50_us | 15981 | 734.4 | 33490 | 2245 | 1670 | 6454 | +109.6% | 10.5 | **P+ better** |
| P+ vs H1 | q5_p50_us | 19875 | 4471 | 42516 | 41608 | 29591 | 24102 | +113.9% | 0.77 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q6_p50_us | 59348 | 5453 | 10199 | 1385 | 3979 | 5879 | -82.8% | 12.4 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 863.4 | 66.8 | 953.8 | 54.5 | 60.9 | 119.3 | +10.5% | 1.48 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 6369 | 1153 | 6307 | 1588 | 1388 | 556.9 | -1.0% | 0.04 | BELOW FLOOR |
| P+ vs H1 | c2_q2_p50_us | 718.8 | 54.0 | 933.1 | 45.8 | 50.1 | 335.6 | +29.8% | 4.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q1_p50_us | 1986 | 156.1 | 1058 | 88.1 | 126.8 | 62.4 | -46.7% | 7.32 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 9694 | 1024 | 5326 | 506.7 | 808.1 | 6292 | -45.1% | 5.41 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q2_p50_us | 1649 | 53.0 | 999.8 | 39.2 | 46.6 | 226.2 | -39.4% | 13.9 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 8515 | 2758 | 6565 | 2583 | 2672 | 3807 | -22.9% | 0.73 | BELOW FLOOR |
| P+ vs H1 | write_p50_us | 1031 | 44.5 | 1751 | 160.3 | 117.7 | 100.2 | +69.9% | 6.12 | **P+ better** |
| P+ vs H1 | write_p99_us | 3279 | 158.6 | 4985 | 591.9 | 433.3 | 1490 | +52.0% | 3.94 | **P+ better** |
| P+ vs H1 | commits_per_s | 831.2 | 30.9 | 493.0 | 26.8 | 28.9 | 108.2 | -40.7% | 11.7 | **P+ better** |
| P+ vs H1 | pss_mib | 43.4 | 0.08 | 419.9 | 5.70 | 4.03 | 36.2 | +868.6% | 93.5 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -749.1 | 533.3 | 19137 | 3632 | 2596 | 25249 | -2654.6% | 7.66 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 181.9 | 14.1 | 141.2 | 7.41 | 11.3 | 40.4 | -22.4% | 3.61 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q1_p99_us | 826.1 | 269.5 | 945.6 | 191.0 | 233.6 | 331.9 | +14.5% | 0.51 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 79056 | 9339 | 240.3 | 21.6 | 6603 | 22143 | -99.7% | 11.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 9918 | 846.8 | 193.0 | 21.2 | 598.9 | 207.1 | -98.1% | 16.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | 55757 | 1827 | 4078 | 479.6 | 1336 | 2187 | -92.7% | 38.7 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 170.4 | 17.5 | 163.4 | 28.5 | 23.6 | 47.1 | -4.1% | 0.29 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q1_p99_us | 2037 | 1410 | 2106 | 493.6 | 1056 | 1597 | +3.4% | 0.07 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 83146 | 9654 | 310.3 | 23.8 | 6826 | 850.6 | -99.6% | 12.1 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 181.9 | 10.8 | 184.2 | 34.1 | 25.3 | 35.9 | +1.2% | 0.09 | BELOW FLOOR |
| N vs H2 | c4_q1_p99_us | 7949 | 1067 | 2529 | 443.0 | 817.1 | 606.0 | -68.2% | 6.63 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 116656 | 9644 | 311.3 | 37.8 | 6820 | 8697 | -99.7% | 17.1 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 412461 | 18514 | 3671 | 827.4 | 13105 | 40763 | -99.1% | 31.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 533.2 | 18.5 | 1657 | 130.9 | 93.5 | 68.6 | +210.7% | 12.0 | **N better** |
| N vs H2 | write_p99_us | 1997 | 1168 | 5391 | 1100 | 1135 | 1811 | +170.0% | 2.99 | no difference |
| N vs H2 | commits_per_s | 1411 | 240.1 | 495.6 | 51.2 | 173.6 | 190.9 | -64.9% | 5.27 | **N better** |
| N vs H2 | pss_mib | 223.5 | 0.80 | 43.4 | 0.07 | 0.57 | 96.0 | -80.6% | 317.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_growth_bytes_per_key_read | 70572 | 7647 | -710.0 | 428.5 | 5416 | 17744 | -101.0% | 13.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p50_us | 698.4 | 23.6 | 141.2 | 7.41 | 17.5 | 55.1 | -79.8% | 31.9 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p99_us | 2522 | 725.7 | 945.6 | 191.0 | 530.6 | 2629 | -62.5% | 2.97 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q2_p50_us | 547.5 | 37.3 | 240.3 | 21.6 | 30.5 | 57.2 | -56.1% | 10.1 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 342.2 | 20.5 | 278.2 | 30.4 | 26.0 | 75.6 | -18.7% | 2.47 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 15981 | 734.4 | 1352 | 169.3 | 532.9 | 818.9 | -91.5% | 27.4 | **H2 better** |
| P+ vs H2 | q5_p50_us | 19875 | 4471 | 193.0 | 21.2 | 3162 | 23790 | -99.0% | 6.23 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q6_p50_us | 59348 | 5453 | 4078 | 479.6 | 3871 | 6192 | -93.1% | 14.3 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 863.4 | 66.8 | 163.4 | 28.5 | 51.3 | 122.2 | -81.1% | 13.6 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q1_p99_us | 6369 | 1153 | 2106 | 493.6 | 887.2 | 682.0 | -66.9% | 4.81 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q2_p50_us | 718.8 | 54.0 | 310.3 | 23.8 | 41.7 | 308.3 | -56.8% | 9.79 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q1_p50_us | 1986 | 156.1 | 184.2 | 34.1 | 113.0 | 53.0 | -90.7% | 16.0 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 9694 | 1024 | 2529 | 443.0 | 789.2 | 6255 | -73.9% | 9.08 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1649 | 53.0 | 311.3 | 37.8 | 46.0 | 196.3 | -81.1% | 29.1 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 8515 | 2758 | 3671 | 827.4 | 2036 | 4280 | -56.9% | 2.38 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | write_p50_us | 1031 | 44.5 | 1657 | 130.9 | 97.7 | 120.2 | +60.7% | 6.40 | **P+ better** |
| P+ vs H2 | write_p99_us | 3279 | 158.6 | 5391 | 1100 | 786.1 | 2033 | +64.4% | 2.69 | no difference |
| P+ vs H2 | commits_per_s | 831.2 | 30.9 | 495.6 | 51.2 | 42.3 | 104.1 | -40.4% | 7.93 | **P+ better** |
| P+ vs H2 | pss_mib | 43.4 | 0.08 | 43.4 | 0.07 | 0.07 | 0.24 | +0.1% | 0.72 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | -749.1 | 533.3 | -710.0 | 428.5 | 483.7 | 2708 | -5.2% | 0.08 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 181.5 | 10.9 | 696.9 | 34.4 | 25.5 | 44.0 | +284.0% | 20.2 | **N better** |
| N vs P+ | q1_p99_us | 710.0 | 68.4 | 2300 | 610.8 | 434.6 | 143.8 | +224.0% | 3.66 | **N better** |
| N vs P+ | q2_p50_us | 76779 | 9027 | 573.2 | 27.4 | 6383 | 1452 | -99.3% | 11.9 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 20436 | 9261 | 36705 | 12339 | 10909 | 2758 | +79.6% | 1.49 | no difference |
| N vs P+ | q6_p50_us | 55141 | 1971 | 57063 | 5427 | 4083 | 20714 | +3.5% | 0.47 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 162.8 | 17.7 | 797.2 | 49.3 | 37.0 | 38.9 | +389.8% | 17.1 | **N better** |
| N vs P+ | c2_q1_p99_us | 2307 | 1853 | 7110 | 1468 | 1672 | 1378 | +208.2% | 2.87 | no difference |
| N vs P+ | c2_q2_p50_us | 87926 | 8037 | 673.6 | 50.5 | 5683 | 17012 | -99.2% | 15.4 | **P+ better** |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c4_q1_p50_us | 187.5 | 20.8 | 1967 | 136.1 | 97.3 | 263.0 | +948.6% | 18.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 7720 | 937.9 | 10422 | 1423 | 1205 | 3579 | +35.0% | 2.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 117703 | 6685 | 1763 | 187.9 | 4729 | 23209 | -98.5% | 24.5 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 400701 | 24541 | 10112 | 1093 | 17371 | 116067 | -97.5% | 22.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 530.8 | 13.3 | 1020 | 88.4 | 63.2 | 137.1 | +92.2% | 7.74 | **N better** |
| N vs P+ | write_p99_us | 2198 | 1202 | 3307 | 175.2 | 859.0 | 603.3 | +50.4% | 1.29 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1444 | 253.5 | 813.3 | 65.8 | 185.2 | 96.9 | -43.7% | 3.40 | **N better** |
| N vs P+ | pss_mib | 248.8 | 1.18 | 43.3 | 0.04 | 0.83 | 1.50 | -82.6% | 247.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 66212 | 7703 | -769.6 | 429.5 | 5456 | 12650 | -101.2% | 12.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 181.5 | 10.9 | 118.8 | 15.7 | 13.5 | 26.0 | -34.5% | 4.64 | **P better** |
| N vs P | q1_p99_us | 710.0 | 68.4 | 622.9 | 95.5 | 83.1 | 63.9 | -12.3% | 1.05 | no difference |
| N vs P | q2_p50_us | 76779 | 9027 | 200.8 | 31.9 | 6383 | 1452 | -99.7% | 12.0 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 20436 | 9261 | 195.9 | 35.3 | 6549 | 424.7 | -99.0% | 3.09 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 55141 | 1971 | 5330 | 935.8 | 1543 | 432.9 | -90.3% | 32.3 | **P better** |
| N vs P | c2_q1_p50_us | 162.8 | 17.7 | 100.0 | 32.7 | 26.3 | 38.9 | -38.6% | 2.39 | no difference |
| N vs P | c2_q1_p99_us | 2307 | 1853 | 1306 | 351.9 | 1334 | 1066 | -43.4% | 0.75 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 87926 | 8037 | 229.9 | 47.1 | 5683 | 17012 | -99.7% | 15.4 | **P better** |
| N vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c4_q1_p50_us | 187.5 | 20.8 | 193.1 | 48.6 | 37.4 | 120.1 | +3.0% | 0.15 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 7720 | 937.9 | 3108 | 477.6 | 744.2 | 3653 | -59.7% | 6.20 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p50_us | 117703 | 6685 | 360.4 | 31.1 | 4727 | 23209 | -99.7% | 24.8 | **P better** |
| N vs P | c4_q2_p99_us | 400701 | 24541 | 6031 | 954.8 | 17367 | 115837 | -98.5% | 22.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 530.8 | 13.3 | 193131 | 7739 | 5472 | 5678 | +36285.9% | 35.2 | **N better** |
| N vs P | write_p99_us | 2198 | 1202 | 243294 | 14938 | 10597 | 20785 | +10968.2% | 22.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1444 | 253.5 | 5.06 | 0.21 | 179.3 | 86.8 | -99.6% | 8.03 | **N better** |
| N vs P | pss_mib | 248.8 | 1.18 | 43.4 | 0.06 | 0.83 | 1.49 | -82.6% | 246.9 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 66212 | 7703 | -760.8 | 286.8 | 5451 | 12624 | -101.1% | 12.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 696.9 | 34.4 | 118.8 | 15.7 | 26.7 | 37.3 | -83.0% | 21.6 | **P better** |
| P+ vs P | q1_p99_us | 2300 | 610.8 | 622.9 | 95.5 | 437.2 | 129.8 | -72.9% | 3.84 | **P better** |
| P+ vs P | q2_p50_us | 573.2 | 27.4 | 200.8 | 31.9 | 29.7 | 8.45 | -65.0% | 12.5 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 326.3 | 34.0 | 248.8 | 59.7 | 48.6 | 94.5 | -23.7% | 1.59 | BELOW FLOOR |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q5_p50_us | 36705 | 12339 | 195.9 | 35.3 | 8725 | 2733 | -99.5% | 4.18 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 57063 | 5427 | 5330 | 935.8 | 3894 | 20709 | -90.7% | 13.3 | **P better** |
| P+ vs P | c2_q1_p50_us | 797.2 | 49.3 | 100.0 | 32.7 | 41.8 | 17.5 | -87.5% | 16.7 | **P better** |
| P+ vs P | c2_q1_p99_us | 7110 | 1468 | 1306 | 351.9 | 1068 | 1227 | -81.6% | 5.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 673.6 | 50.5 | 229.9 | 47.1 | 48.8 | 54.4 | -65.9% | 9.08 | **P better** |
| P+ vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c4_q1_p50_us | 1967 | 136.1 | 193.1 | 48.6 | 102.2 | 280.2 | -90.2% | 17.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 10422 | 1423 | 3108 | 477.6 | 1061 | 1317 | -70.2% | 6.89 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1763 | 187.9 | 360.4 | 31.1 | 134.7 | 57.0 | -79.6% | 10.4 | **P better** |
| P+ vs P | c4_q2_p99_us | 10112 | 1093 | 6031 | 954.8 | 1026 | 8176 | -40.4% | 3.98 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 1020 | 88.4 | 193131 | 7739 | 5473 | 5679 | +18831.6% | 35.1 | **P+ better** |
| P+ vs P | write_p99_us | 3307 | 175.2 | 243294 | 14938 | 10563 | 20784 | +7257.7% | 22.7 | **P+ better** |
| P+ vs P | commits_per_s | 813.3 | 65.8 | 5.06 | 0.21 | 46.6 | 43.1 | -99.4% | 17.4 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.04 | 43.4 | 0.06 | 0.05 | 0.15 | +0.2% | 2.06 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -769.6 | 429.5 | -760.8 | 286.8 | 365.2 | 2496 | -1.1% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 118.8 | 15.7 | 79.4 | 8.11 | 12.5 | 11.5 | -33.2% | 3.16 | **M better** |
| P vs M | q1_p99_us | 622.9 | 95.5 | 577.2 | 87.5 | 91.6 | 80.0 | -7.3% | 0.50 | BELOW FLOOR |
| P vs M | q2_p50_us | 200.8 | 31.9 | 185.3 | 25.6 | 29.0 | 15.5 | -7.7% | 0.54 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 248.8 | 59.7 | 205.2 | 32.6 | 48.1 | 50.2 | -17.5% | 0.91 | BELOW FLOOR |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q5_p50_us | 195.9 | 35.3 | 170.9 | 24.6 | 30.4 | 168.6 | -12.8% | 0.82 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | 5330 | 935.8 | 5912 | 290.2 | 692.8 | 187.9 | +10.9% | 0.84 | no difference |
| P vs M | c2_q1_p50_us | 100.0 | 32.7 | 129.6 | 55.8 | 45.7 | 66.7 | +29.7% | 0.65 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1306 | 351.9 | 2529 | 679.2 | 540.9 | 642.8 | +93.7% | 2.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 229.9 | 47.1 | 254.9 | 43.8 | 45.5 | 106.2 | +10.9% | 0.55 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c4_q1_p50_us | 193.1 | 48.6 | 178.5 | 33.1 | 41.6 | 125.4 | -7.6% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 3108 | 477.6 | 3047 | 638.7 | 563.9 | 1937 | -2.0% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p50_us | 360.4 | 31.1 | 340.7 | 35.5 | 33.4 | 138.2 | -5.5% | 0.59 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p99_us | 6031 | 954.8 | 4962 | 1385 | 1190 | 2609 | -17.7% | 0.90 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 193131 | 7739 | 194887 | 3496 | 6005 | 6116 | +0.9% | 0.29 | BELOW FLOOR |
| P vs M | write_p99_us | 243294 | 14938 | 239995 | 11613 | 13379 | 21198 | -1.4% | 0.25 | BELOW FLOOR |
| P vs M | commits_per_s | 5.06 | 0.21 | 4.95 | 0.12 | 0.17 | 0.21 | -2.2% | 0.66 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.06 | 43.5 | 0.24 | 0.17 | 0.06 | +0.3% | 0.68 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -760.8 | 286.8 | -719.1 | 332.6 | 310.5 | 2305 | -5.5% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 181.5 | 10.9 | 435.4 | 27.9 | 21.2 | 33.5 | +139.9% | 12.0 | **N better** |
| N vs H3 | q1_p99_us | 710.0 | 68.4 | 1546 | 269.0 | 196.3 | 579.9 | +117.7% | 4.26 | **N better** |
| N vs H3 | q2_p50_us | 76779 | 9027 | 461.5 | 68.0 | 6383 | 1452 | -99.4% | 12.0 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 20436 | 9261 | 38013 | 10338 | 9814 | 1183 | +86.0% | 1.79 | no difference |
| N vs H3 | q6_p50_us | 55141 | 1971 | 65273 | 3795 | 3024 | 9776 | +18.4% | 3.35 | **N better** |
| N vs H3 | c2_q1_p50_us | 162.8 | 17.7 | 467.5 | 46.9 | 35.5 | 47.4 | +187.2% | 8.59 | **N better** |
| N vs H3 | c2_q1_p99_us | 2307 | 1853 | 2542 | 885.2 | 1452 | 1081 | +10.2% | 0.16 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 87926 | 8037 | 443.4 | 30.4 | 5683 | 17012 | -99.5% | 15.4 | **H3 better** |
| N vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c4_q1_p50_us | 187.5 | 20.8 | 783.1 | 111.2 | 80.0 | 80.6 | +317.6% | 7.45 | **N better** |
| N vs H3 | c4_q1_p99_us | 7720 | 937.9 | 3620 | 760.8 | 854.0 | 3938 | -53.1% | 4.80 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 117703 | 6685 | 776.0 | 86.4 | 4727 | 23213 | -99.3% | 24.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 400701 | 24541 | 3763 | 1161 | 17373 | 115809 | -99.1% | 22.8 | **H3 better** |
| N vs H3 | write_p50_us | 530.8 | 13.3 | 1466 | 133.8 | 95.1 | 285.9 | +176.2% | 9.83 | **N better** |
| N vs H3 | write_p99_us | 2198 | 1202 | 4826 | 1146 | 1174 | 6781 | +119.6% | 2.24 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1444 | 253.5 | 568.0 | 38.3 | 181.3 | 157.1 | -60.7% | 4.83 | **N better** |
| N vs H3 | pss_mib | 248.8 | 1.18 | 109.4 | 5.59 | 4.04 | 1.49 | -56.0% | 34.5 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 66212 | 7703 | 19981 | 2877 | 5815 | 40977 | -69.8% | 7.95 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 696.9 | 34.4 | 435.4 | 27.9 | 31.3 | 42.8 | -37.5% | 8.36 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2300 | 610.8 | 1546 | 269.0 | 472.0 | 590.8 | -32.8% | 1.60 | no difference |
| P+ vs H3 | q2_p50_us | 573.2 | 27.4 | 461.5 | 68.0 | 51.8 | 8.49 | -19.5% | 2.16 | no difference |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 326.3 | 34.0 | 392.9 | 49.7 | 42.6 | 100.9 | +20.4% | 1.56 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q5_p50_us | 36705 | 12339 | 38013 | 10338 | 11383 | 2948 | +3.6% | 0.11 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 57063 | 5427 | 65273 | 3795 | 4683 | 22897 | +14.4% | 1.75 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 797.2 | 49.3 | 467.5 | 46.9 | 48.1 | 32.2 | -41.4% | 6.85 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 7110 | 1468 | 2542 | 885.2 | 1212 | 1241 | -64.2% | 3.77 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 673.6 | 50.5 | 443.4 | 30.4 | 41.7 | 52.3 | -34.2% | 5.52 | **H3 better** |
| P+ vs H3 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c4_q1_p50_us | 1967 | 136.1 | 783.1 | 111.2 | 124.3 | 265.6 | -60.2% | 9.52 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 10422 | 1423 | 3620 | 760.8 | 1141 | 1975 | -65.3% | 5.96 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1763 | 187.9 | 776.0 | 86.4 | 146.3 | 425.1 | -56.0% | 6.75 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 10112 | 1093 | 3763 | 1161 | 1128 | 7767 | -62.8% | 5.63 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1020 | 88.4 | 1466 | 133.8 | 113.4 | 316.6 | +43.7% | 3.93 | **P+ better** |
| P+ vs H3 | write_p99_us | 3307 | 175.2 | 4826 | 1146 | 819.8 | 6778 | +46.0% | 1.85 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 813.3 | 65.8 | 568.0 | 38.3 | 53.9 | 137.8 | -30.2% | 4.56 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.04 | 109.4 | 5.59 | 3.95 | 0.15 | +153.0% | 16.7 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -769.6 | 429.5 | 19981 | 2877 | 2057 | 39064 | -2696.4% | 10.1 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 181.5 | 10.9 | 1377 | 70.8 | 50.6 | 155.9 | +658.4% | 23.6 | **N better** |
| N vs T | q1_p99_us | 710.0 | 68.4 | 7454 | 4460 | 3154 | 3027 | +949.9% | 2.14 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | q2_p50_us | 76779 | 9027 | 1389 | 75.0 | 6384 | 1458 | -98.2% | 11.8 | **T better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 162.8 | 17.7 | 1922 | 76.9 | 55.8 | 160.3 | +1080.8% | 31.5 | **N better** |
| N vs T | c2_q1_p99_us | 2307 | 1853 | 6497 | 1495 | 1684 | 876.9 | +181.6% | 2.49 | no difference |
| N vs T | c2_q2_p50_us | 87926 | 8037 | 1929 | 96.0 | 5684 | 17016 | -97.8% | 15.1 | **T better** |
| N vs T | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c4_q1_p50_us | 187.5 | 20.8 | 2411 | 99.3 | 71.7 | 53.5 | +1185.8% | 31.0 | **N better** |
| N vs T | c4_q1_p99_us | 7720 | 937.9 | 4973 | 1171 | 1061 | 8489 | -35.6% | 2.59 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c4_q2_p50_us | 117703 | 6685 | 2462 | 163.2 | 4728 | 23209 | -97.9% | 24.4 | **T better** |
| N vs T | c4_q2_p99_us | 400701 | 24541 | 5205 | 1667 | 17393 | 115825 | -98.7% | 22.7 | **T better** |
| N vs T | write_p50_us | 530.8 | 13.3 | 1275 | 43.7 | 32.3 | 85.0 | +140.2% | 23.0 | **N better** |
| N vs T | write_p99_us | 2198 | 1202 | 3372 | 1158 | 1180 | 451.6 | +53.4% | 0.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | commits_per_s | 1444 | 253.5 | 682.4 | 75.8 | 187.1 | 104.8 | -52.7% | 4.07 | **N better** |
| N vs T | pss_mib | 248.8 | 1.18 | 2580 | 0.16 | 0.84 | 1.51 | +937.1% | 2780 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 66212 | 7703 | -129.4 | 83.1 | 5447 | 12520 | -100.2% | 12.2 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | q1_p50_us | 696.9 | 34.4 | 1377 | 70.8 | 55.6 | 158.2 | +97.5% | 12.2 | **P+ better** |
| P+ vs T | q1_p99_us | 2300 | 610.8 | 7454 | 4460 | 3183 | 3029 | +224.0% | 1.62 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | q2_p50_us | 573.2 | 27.4 | 1389 | 75.0 | 56.5 | 133.8 | +142.3% | 14.4 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 797.2 | 49.3 | 1922 | 76.9 | 64.6 | 156.5 | +141.1% | 17.4 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 7110 | 1468 | 6497 | 1495 | 1482 | 1067 | -8.6% | 0.41 | BELOW FLOOR |
| P+ vs T | c2_q2_p50_us | 673.6 | 50.5 | 1929 | 96.0 | 76.7 | 357.7 | +186.3% | 16.4 | **P+ better** |
| P+ vs T | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c4_q1_p50_us | 1967 | 136.1 | 2411 | 99.3 | 119.1 | 258.7 | +22.6% | 3.73 | **P+ better** |
| P+ vs T | c4_q1_p99_us | 10422 | 1423 | 4973 | 1171 | 1303 | 7775 | -52.3% | 4.18 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c4_q2_p50_us | 1763 | 187.9 | 2462 | 163.2 | 176.0 | 150.5 | +39.7% | 3.97 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 10112 | 1093 | 5205 | 1667 | 1410 | 8008 | -48.5% | 3.48 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | write_p50_us | 1020 | 88.4 | 1275 | 43.7 | 69.8 | 160.4 | +25.0% | 3.65 | **P+ better** |
| P+ vs T | write_p99_us | 3307 | 175.2 | 3372 | 1158 | 828.3 | 404.1 | +2.0% | 0.08 | BELOW FLOOR |
| P+ vs T | commits_per_s | 813.3 | 65.8 | 682.4 | 75.8 | 71.0 | 72.8 | -16.1% | 1.84 | no difference |
| P+ vs T | pss_mib | 43.3 | 0.04 | 2580 | 0.16 | 0.12 | 0.25 | +5865.7% | 21844 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -769.6 | 429.5 | -129.4 | 83.1 | 309.3 | 1902 | -83.2% | 2.07 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 435.4 | 27.9 | 1377 | 70.8 | 53.8 | 155.6 | +216.1% | 17.5 | **H3 better** |
| H3 vs T | q1_p99_us | 1546 | 269.0 | 7454 | 4460 | 3160 | 3081 | +382.2% | 1.87 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | q2_p50_us | 461.5 | 68.0 | 1389 | 75.0 | 71.6 | 134.0 | +200.9% | 13.0 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 467.5 | 46.9 | 1922 | 76.9 | 63.7 | 158.8 | +311.2% | 22.8 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 2542 | 885.2 | 6497 | 1495 | 1229 | 640.1 | +155.6% | 3.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q2_p50_us | 443.4 | 30.4 | 1929 | 96.0 | 71.2 | 353.9 | +334.9% | 20.8 | **H3 better** |
| H3 vs T | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c4_q1_p50_us | 783.1 | 111.2 | 2411 | 99.3 | 105.4 | 65.1 | +207.9% | 15.5 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 3620 | 760.8 | 4973 | 1171 | 987.6 | 7947 | +37.4% | 1.37 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c4_q2_p50_us | 776.0 | 86.4 | 2462 | 163.2 | 130.6 | 444.0 | +217.3% | 12.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p99_us | 3763 | 1161 | 5205 | 1667 | 1437 | 2035 | +38.3% | 1.00 | BELOW FLOOR |
| H3 vs T | write_p50_us | 1466 | 133.8 | 1275 | 43.7 | 99.6 | 297.8 | -13.0% | 1.92 | BELOW FLOOR |
| H3 vs T | write_p99_us | 4826 | 1146 | 3372 | 1158 | 1152 | 6766 | -30.1% | 1.26 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | commits_per_s | 568.0 | 38.3 | 682.4 | 75.8 | 60.0 | 143.4 | +20.2% | 1.91 | BELOW FLOOR |
| H3 vs T | pss_mib | 109.4 | 5.59 | 2580 | 0.16 | 3.96 | 0.22 | +2257.8% | 624.6 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 19981 | 2877 | -129.4 | 83.1 | 2036 | 39022 | -100.6% | 9.88 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 181.5 | 10.9 | 1004 | 80.8 | 57.7 | 195.2 | +453.3% | 14.3 | **N better** |
| N vs H1 | q1_p99_us | 710.0 | 68.4 | 5115 | 1672 | 1184 | 1707 | +620.4% | 3.72 | **N better** |
| N vs H1 | q2_p50_us | 76779 | 9027 | 869.1 | 109.7 | 6384 | 1452 | -98.9% | 11.9 | **H1 better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 20436 | 9261 | 42721 | 42017 | 30424 | 7091 | +109.0% | 0.73 | no difference |
| N vs H1 | q6_p50_us | 55141 | 1971 | 10593 | 481.4 | 1434 | 485.0 | -80.8% | 31.1 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 162.8 | 17.7 | 957.6 | 93.8 | 67.5 | 36.9 | +488.3% | 11.8 | **N better** |
| N vs H1 | c2_q1_p99_us | 2307 | 1853 | 6297 | 1747 | 1800 | 1989 | +173.0% | 2.22 | no difference |
| N vs H1 | c2_q2_p50_us | 87926 | 8037 | 1024 | 91.4 | 5684 | 17012 | -98.8% | 15.3 | **H1 better** |
| N vs H1 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | c4_q1_p50_us | 187.5 | 20.8 | 981.0 | 93.7 | 67.9 | 145.5 | +423.1% | 11.7 | **N better** |
| N vs H1 | c4_q1_p99_us | 7720 | 937.9 | 5240 | 1081 | 1012 | 3640 | -32.1% | 2.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p50_us | 117703 | 6685 | 955.1 | 63.8 | 4727 | 23209 | -99.2% | 24.7 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 400701 | 24541 | 6449 | 2199 | 17423 | 115856 | -98.4% | 22.6 | **H1 better** |
| N vs H1 | write_p50_us | 530.8 | 13.3 | 1692 | 75.7 | 54.3 | 108.0 | +218.8% | 21.4 | **N better** |
| N vs H1 | write_p99_us | 2198 | 1202 | 5946 | 1134 | 1169 | 1754 | +170.5% | 3.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | commits_per_s | 1444 | 253.5 | 500.5 | 20.9 | 179.9 | 115.5 | -65.3% | 5.24 | **N better** |
| N vs H1 | pss_mib | 248.8 | 1.18 | 419.0 | 12.0 | 8.55 | 10.0 | +68.4% | 19.9 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 66212 | 7703 | 14593 | 1450 | 5543 | 19891 | -78.0% | 9.31 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q1_p50_us | 696.9 | 34.4 | 1004 | 80.8 | 62.1 | 197.1 | +44.1% | 4.95 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2300 | 610.8 | 5115 | 1672 | 1259 | 1710 | +122.3% | 2.24 | no difference |
| P+ vs H1 | q2_p50_us | 573.2 | 27.4 | 869.1 | 109.7 | 79.9 | 24.9 | +51.6% | 3.70 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 326.3 | 34.0 | 886.5 | 46.7 | 40.9 | 117.3 | +171.7% | 13.7 | **P+ better** |
| P+ vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q5_p50_us | 36705 | 12339 | 42721 | 42017 | 30965 | 7588 | +16.4% | 0.19 | BELOW FLOOR |
| P+ vs H1 | q6_p50_us | 57063 | 5427 | 10593 | 481.4 | 3852 | 20710 | -81.4% | 12.1 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 797.2 | 49.3 | 957.6 | 93.8 | 74.9 | 12.4 | +20.1% | 2.14 | no difference |
| P+ vs H1 | c2_q1_p99_us | 7110 | 1468 | 6297 | 1747 | 1613 | 2080 | -11.4% | 0.50 | BELOW FLOOR |
| P+ vs H1 | c2_q2_p50_us | 673.6 | 50.5 | 1024 | 91.4 | 73.9 | 140.6 | +52.1% | 4.75 | **P+ better** |
| P+ vs H1 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | c4_q1_p50_us | 1967 | 136.1 | 981.0 | 93.7 | 116.8 | 292.0 | -50.1% | 8.43 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 10422 | 1423 | 5240 | 1081 | 1264 | 1279 | -49.7% | 4.10 | **H1 better** |
| P+ vs H1 | c4_q2_p50_us | 1763 | 187.9 | 955.1 | 63.8 | 140.3 | 56.5 | -45.8% | 5.76 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 10112 | 1093 | 6449 | 2199 | 1736 | 8438 | -36.2% | 2.11 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | write_p50_us | 1020 | 88.4 | 1692 | 75.7 | 82.3 | 173.7 | +65.9% | 8.16 | **P+ better** |
| P+ vs H1 | write_p99_us | 3307 | 175.2 | 5946 | 1134 | 811.6 | 1742 | +79.8% | 3.25 | **P+ better** |
| P+ vs H1 | commits_per_s | 813.3 | 65.8 | 500.5 | 20.9 | 48.8 | 87.5 | -38.5% | 6.41 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.04 | 419.0 | 12.0 | 8.51 | 9.90 | +868.7% | 44.1 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -769.6 | 429.5 | 14593 | 1450 | 1069 | 15574 | -1996.3% | 14.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 181.5 | 10.9 | 140.0 | 13.0 | 12.0 | 25.2 | -22.9% | 3.46 | **H2 better** |
| N vs H2 | q1_p99_us | 710.0 | 68.4 | 769.0 | 152.4 | 118.1 | 503.7 | +8.3% | 0.50 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 76779 | 9027 | 244.9 | 21.9 | 6383 | 1453 | -99.7% | 12.0 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 20436 | 9261 | 192.0 | 19.3 | 6549 | 401.1 | -99.1% | 3.09 | **H2 better** |
| N vs H2 | q6_p50_us | 55141 | 1971 | 4316 | 826.3 | 1511 | 787.7 | -92.2% | 33.6 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 162.8 | 17.7 | 92.5 | 33.1 | 26.6 | 37.7 | -43.2% | 2.65 | no difference |
| N vs H2 | c2_q1_p99_us | 2307 | 1853 | 1452 | 217.3 | 1319 | 901.6 | -37.1% | 0.65 | BELOW FLOOR |
| N vs H2 | c2_q2_p50_us | 87926 | 8037 | 239.6 | 30.0 | 5683 | 17012 | -99.7% | 15.4 | **H2 better** |
| N vs H2 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c4_q1_p50_us | 187.5 | 20.8 | 199.4 | 52.7 | 40.0 | 67.2 | +6.3% | 0.30 | BELOW FLOOR |
| N vs H2 | c4_q1_p99_us | 7720 | 937.9 | 2973 | 328.5 | 702.7 | 3702 | -61.5% | 6.75 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q2_p50_us | 117703 | 6685 | 328.5 | 42.9 | 4727 | 23209 | -99.7% | 24.8 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p99_us | 400701 | 24541 | 5309 | 1236 | 17375 | 115811 | -98.7% | 22.8 | **H2 better** |
| N vs H2 | write_p50_us | 530.8 | 13.3 | 1495 | 101.6 | 72.4 | 179.9 | +181.7% | 13.3 | **N better** |
| N vs H2 | write_p99_us | 2198 | 1202 | 5481 | 850.9 | 1041 | 775.6 | +149.3% | 3.15 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1444 | 253.5 | 524.4 | 55.9 | 183.6 | 87.2 | -63.7% | 5.01 | **N better** |
| N vs H2 | pss_mib | 248.8 | 1.18 | 43.5 | 0.02 | 0.83 | 1.49 | -82.5% | 247.0 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 66212 | 7703 | -776.1 | 146.3 | 5448 | 12635 | -101.2% | 12.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p50_us | 696.9 | 34.4 | 140.0 | 13.0 | 26.0 | 36.7 | -79.9% | 21.4 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2300 | 610.8 | 769.0 | 152.4 | 445.2 | 516.2 | -66.6% | 3.44 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p50_us | 573.2 | 27.4 | 244.9 | 21.9 | 24.8 | 53.5 | -57.3% | 13.2 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 326.3 | 34.0 | 263.1 | 38.1 | 36.1 | 100.7 | -19.4% | 1.75 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q5_p50_us | 36705 | 12339 | 192.0 | 19.3 | 8725 | 2730 | -99.5% | 4.18 | **H2 better** |
| P+ vs H2 | q6_p50_us | 57063 | 5427 | 4316 | 826.3 | 3882 | 20720 | -92.4% | 13.6 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 797.2 | 49.3 | 92.5 | 33.1 | 42.0 | 14.6 | -88.4% | 16.8 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 7110 | 1468 | 1452 | 217.3 | 1050 | 1087 | -79.6% | 5.39 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 673.6 | 50.5 | 239.6 | 30.0 | 41.6 | 77.1 | -64.4% | 10.4 | **H2 better** |
| P+ vs H2 | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c4_q1_p50_us | 1967 | 136.1 | 199.4 | 52.7 | 103.2 | 261.9 | -89.9% | 17.1 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 10422 | 1423 | 2973 | 328.5 | 1032 | 1448 | -71.5% | 7.22 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p50_us | 1763 | 187.9 | 328.5 | 42.9 | 136.3 | 121.1 | -81.4% | 10.5 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p99_us | 10112 | 1093 | 5309 | 1236 | 1167 | 7803 | -47.5% | 4.12 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 1020 | 88.4 | 1495 | 101.6 | 95.2 | 225.5 | +46.6% | 4.99 | **P+ better** |
| P+ vs H2 | write_p99_us | 3307 | 175.2 | 5481 | 850.9 | 614.3 | 749.0 | +65.7% | 3.54 | **P+ better** |
| P+ vs H2 | commits_per_s | 813.3 | 65.8 | 524.4 | 55.9 | 61.1 | 43.8 | -35.5% | 4.73 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.04 | 43.5 | 0.02 | 0.03 | 0.13 | +0.6% | 8.40 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -769.6 | 429.5 | -776.1 | 146.3 | 320.8 | 2555 | +0.8% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 194.7 | 12.7 | 747.6 | 55.8 | 40.5 | 17.5 | +284.0% | 13.7 | **N better** |
| N vs P+ | q1_p99_us | 657.8 | 156.0 | 2477 | 817.6 | 588.5 | 1183 | +276.6% | 3.09 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 77307 | 6735 | 525.3 | 29.2 | 4763 | 8410 | -99.3% | 16.1 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 12777 | 3996 | 29924 | 10911 | 8217 | 9317 | +134.2% | 2.09 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q6_p50_us | 59721 | 1892 | 60986 | 4664 | 3559 | 7176 | +2.1% | 0.36 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 159.6 | 16.9 | 938.8 | 105.4 | 75.5 | 14.6 | +488.3% | 10.3 | **N better** |
| N vs P+ | c2_q1_p99_us | 2822 | 1260 | 5541 | 2197 | 1791 | 1428 | +96.4% | 1.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 88571 | 6779 | 708.4 | 41.8 | 4794 | 5913 | -99.2% | 18.3 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 181.7 | 26.3 | 2032 | 124.8 | 90.2 | 44.1 | +1018.0% | 20.5 | **N better** |
| N vs P+ | c4_q1_p99_us | 7563 | 599.9 | 11372 | 3262 | 2345 | 3197 | +50.4% | 1.62 | no difference |
| N vs P+ | c4_q2_p50_us | 119671 | 4501 | 1828 | 126.2 | 3184 | 11724 | -98.5% | 37.0 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 412769 | 23267 | 8945 | 3695 | 16659 | 54358 | -97.8% | 24.2 | **P+ better** |
| N vs P+ | write_p50_us | 622.4 | 39.5 | 1117 | 107.1 | 80.7 | 74.0 | +79.5% | 6.13 | **N better** |
| N vs P+ | write_p99_us | 1999 | 936.7 | 3408 | 317.7 | 699.4 | 2779 | +70.5% | 2.01 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1299 | 237.7 | 744.9 | 36.9 | 170.1 | 359.9 | -42.7% | 3.26 | **N better** |
| N vs P+ | pss_mib | 178.5 | 40.5 | 47.2 | 3.97 | 28.8 | 0.54 | -73.6% | 4.56 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 50381 | 4218 | 465.6 | 1790 | 3240 | 69007 | -99.1% | 15.4 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 194.7 | 12.7 | 116.4 | 20.9 | 17.3 | 46.2 | -40.2% | 4.53 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q1_p99_us | 657.8 | 156.0 | 612.6 | 158.8 | 157.4 | 641.2 | -6.9% | 0.29 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 77307 | 6735 | 193.8 | 20.5 | 4763 | 8410 | -99.7% | 16.2 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 12777 | 3996 | 173.7 | 20.9 | 2826 | 8593 | -98.6% | 4.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q6_p50_us | 59721 | 1892 | 5953 | 194.5 | 1345 | 2923 | -90.0% | 40.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p50_us | 159.6 | 16.9 | 118.2 | 40.2 | 30.9 | 11.1 | -25.9% | 1.34 | no difference |
| N vs P | c2_q1_p99_us | 2822 | 1260 | 1754 | 655.1 | 1004 | 4892 | -37.8% | 1.06 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 88571 | 6779 | 293.1 | 38.3 | 4794 | 5914 | -99.7% | 18.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 181.7 | 26.3 | 179.9 | 42.1 | 35.1 | 97.2 | -1.0% | 0.05 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 7563 | 599.9 | 2991 | 341.4 | 488.1 | 2140 | -60.4% | 9.37 | **P better** |
| N vs P | c4_q2_p50_us | 119671 | 4501 | 347.4 | 48.5 | 3183 | 11720 | -99.7% | 37.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 412769 | 23267 | 3843 | 851.1 | 16464 | 54260 | -99.1% | 24.8 | **P better** |
| N vs P | write_p50_us | 622.4 | 39.5 | 195736 | 4718 | 3336 | 2182 | +31346.2% | 58.5 | **N better** |
| N vs P | write_p99_us | 1999 | 936.7 | 248249 | 21601 | 15289 | 33795 | +12316.2% | 16.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1299 | 237.7 | 4.93 | 0.17 | 168.1 | 359.4 | -99.6% | 7.70 | **N better** |
| N vs P | pss_mib | 178.5 | 40.5 | 46.2 | 2.88 | 28.7 | 0.54 | -74.1% | 4.60 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 50381 | 4218 | 188.8 | 1438 | 3151 | 69003 | -99.6% | 15.9 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 747.6 | 55.8 | 116.4 | 20.9 | 42.2 | 45.6 | -84.4% | 15.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p99_us | 2477 | 817.6 | 612.6 | 158.8 | 588.9 | 1156 | -75.3% | 3.17 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 525.3 | 29.2 | 193.8 | 20.5 | 25.2 | 93.4 | -63.1% | 13.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 314.8 | 15.3 | 262.5 | 22.3 | 19.1 | 77.7 | -16.6% | 2.74 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 16410 | 1216 | 1068 | 52.7 | 860.8 | 557.7 | -93.5% | 17.8 | **P better** |
| P+ vs P | q5_p50_us | 29924 | 10911 | 173.7 | 20.9 | 7715 | 3600 | -99.4% | 3.86 | **P better** |
| P+ vs P | q6_p50_us | 60986 | 4664 | 5953 | 194.5 | 3301 | 7488 | -90.2% | 16.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p50_us | 938.8 | 105.4 | 118.2 | 40.2 | 79.7 | 11.7 | -87.4% | 10.3 | **P better** |
| P+ vs P | c2_q1_p99_us | 5541 | 2197 | 1754 | 655.1 | 1621 | 5040 | -68.3% | 2.34 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 708.4 | 41.8 | 293.1 | 38.3 | 40.1 | 117.9 | -58.6% | 10.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 2032 | 124.8 | 179.9 | 42.1 | 93.1 | 96.7 | -91.1% | 19.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 11372 | 3262 | 2991 | 341.4 | 2319 | 2423 | -73.7% | 3.61 | **P better** |
| P+ vs P | c4_q2_p50_us | 1828 | 126.2 | 347.4 | 48.5 | 95.6 | 334.7 | -81.0% | 15.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 8945 | 3695 | 3843 | 851.1 | 2681 | 3471 | -57.0% | 1.90 | no difference |
| P+ vs P | write_p50_us | 1117 | 107.1 | 195736 | 4718 | 3337 | 2182 | +17419.8% | 58.3 | **P+ better** |
| P+ vs P | write_p99_us | 3408 | 317.7 | 248249 | 21601 | 15276 | 33845 | +7184.2% | 16.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 744.9 | 36.9 | 4.93 | 0.17 | 26.1 | 19.7 | -99.3% | 28.3 | **P+ better** |
| P+ vs P | pss_mib | 47.2 | 3.97 | 46.2 | 2.88 | 3.47 | 0.12 | -2.1% | 0.29 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 465.6 | 1790 | 188.8 | 1438 | 1624 | 2655 | -59.4% | 0.17 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 116.4 | 20.9 | 114.7 | 38.5 | 31.0 | 85.6 | -1.4% | 0.05 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q1_p99_us | 612.6 | 158.8 | 876.1 | 297.6 | 238.5 | 562.8 | +43.0% | 1.11 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 193.8 | 20.5 | 215.5 | 24.4 | 22.5 | 90.6 | +11.2% | 0.96 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 262.5 | 22.3 | 246.4 | 36.0 | 29.9 | 94.1 | -6.1% | 0.54 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 1068 | 52.7 | 1082 | 55.5 | 54.1 | 45.3 | +1.3% | 0.26 | BELOW FLOOR |
| P vs M | q5_p50_us | 173.7 | 20.9 | 187.1 | 44.5 | 34.8 | 92.3 | +7.8% | 0.39 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | 5953 | 194.5 | 6366 | 109.1 | 157.7 | 2577 | +6.9% | 2.62 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p50_us | 118.2 | 40.2 | 117.3 | 46.3 | 43.4 | 7.50 | -0.8% | 0.02 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1754 | 655.1 | 2268 | 335.0 | 520.3 | 4863 | +29.3% | 0.99 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 293.1 | 38.3 | 265.4 | 55.2 | 47.5 | 114.7 | -9.4% | 0.58 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 179.9 | 42.1 | 199.5 | 19.6 | 32.8 | 92.9 | +10.9% | 0.60 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2991 | 341.4 | 3175 | 308.4 | 325.3 | 696.3 | +6.2% | 0.57 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 347.4 | 48.5 | 346.6 | 53.5 | 51.1 | 146.8 | -0.2% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 3843 | 851.1 | 4925 | 1364 | 1137 | 956.8 | +28.1% | 0.95 | no difference |
| P vs M | write_p50_us | 195736 | 4718 | 192141 | 6573 | 5721 | 2913 | -1.8% | 0.63 | no difference |
| P vs M | write_p99_us | 248249 | 21601 | 256447 | 16768 | 19336 | 51085 | +3.3% | 0.42 | BELOW FLOOR |
| P vs M | commits_per_s | 4.93 | 0.17 | 4.94 | 0.16 | 0.16 | 0.10 | +0.3% | 0.08 | BELOW FLOOR |
| P vs M | pss_mib | 46.2 | 2.88 | 46.7 | 3.38 | 3.14 | 0.08 | +1.1% | 0.16 | no difference |
| P vs M | pss_growth_bytes_per_key_read | 188.8 | 1438 | 375.6 | 1638 | 1541 | 2560 | +98.9% | 0.12 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 194.7 | 12.7 | 470.4 | 38.6 | 28.7 | 30.7 | +141.6% | 9.60 | **N better** |
| N vs H3 | q1_p99_us | 657.8 | 156.0 | 1586 | 351.7 | 272.1 | 506.5 | +141.1% | 3.41 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 77307 | 6735 | 444.6 | 75.3 | 4763 | 8410 | -99.4% | 16.1 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 12777 | 3996 | 28483 | 10524 | 7960 | 30265 | +122.9% | 1.97 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q6_p50_us | 59721 | 1892 | 66790 | 2944 | 2474 | 2671 | +11.8% | 2.86 | no difference |
| N vs H3 | c2_q1_p50_us | 159.6 | 16.9 | 399.6 | 37.3 | 29.0 | 182.8 | +150.4% | 8.29 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 2822 | 1260 | 1706 | 515.8 | 962.8 | 2899 | -39.5% | 1.16 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 88571 | 6779 | 397.1 | 37.2 | 4794 | 5915 | -99.6% | 18.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 181.7 | 26.3 | 733.7 | 44.0 | 36.2 | 71.8 | +303.7% | 15.2 | **N better** |
| N vs H3 | c4_q1_p99_us | 7563 | 599.9 | 2834 | 642.3 | 621.5 | 2737 | -62.5% | 7.61 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 119671 | 4501 | 769.3 | 49.6 | 3183 | 11720 | -99.4% | 37.4 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 412769 | 23267 | 2608 | 444.8 | 16456 | 54428 | -99.4% | 24.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 622.4 | 39.5 | 1569 | 92.2 | 70.9 | 134.8 | +152.1% | 13.4 | **N better** |
| N vs H3 | write_p99_us | 1999 | 936.7 | 5193 | 1048 | 993.9 | 1595 | +159.7% | 3.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1299 | 237.7 | 525.2 | 72.3 | 175.7 | 371.3 | -59.6% | 4.41 | **N better** |
| N vs H3 | pss_mib | 178.5 | 40.5 | 107.6 | 1.98 | 28.7 | 0.55 | -39.7% | 2.47 | no difference |
| N vs H3 | pss_growth_bytes_per_key_read | 50381 | 4218 | 19632 | 3788 | 4009 | 80587 | -61.0% | 7.67 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 747.6 | 55.8 | 470.4 | 38.6 | 48.0 | 29.7 | -37.1% | 5.78 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2477 | 817.6 | 1586 | 351.7 | 629.3 | 1087 | -36.0% | 1.42 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 525.3 | 29.2 | 444.6 | 75.3 | 57.1 | 91.4 | -15.4% | 1.41 | BELOW FLOOR |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 314.8 | 15.3 | 405.5 | 45.2 | 33.8 | 85.6 | +28.8% | 2.69 | no difference |
| P+ vs H3 | q4_p50_us | 16410 | 1216 | 15912 | 568.2 | 949.2 | 759.5 | -3.0% | 0.52 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 29924 | 10911 | 28483 | 10524 | 10719 | 29242 | -4.8% | 0.13 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q6_p50_us | 60986 | 4664 | 66790 | 2944 | 3900 | 7393 | +9.5% | 1.49 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 938.8 | 105.4 | 399.6 | 37.3 | 79.0 | 182.9 | -57.4% | 6.82 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 5541 | 2197 | 1706 | 515.8 | 1596 | 3143 | -69.2% | 2.40 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 708.4 | 41.8 | 397.1 | 37.2 | 39.5 | 162.4 | -44.0% | 7.87 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2032 | 124.8 | 733.7 | 44.0 | 93.6 | 71.3 | -63.9% | 13.9 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 11372 | 3262 | 2834 | 642.3 | 2351 | 2964 | -75.1% | 3.63 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1828 | 126.2 | 769.3 | 49.6 | 95.9 | 320.6 | -57.9% | 11.0 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8945 | 3695 | 2608 | 444.8 | 2632 | 5508 | -70.8% | 2.41 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1117 | 107.1 | 1569 | 92.2 | 99.9 | 144.8 | +40.5% | 4.53 | **P+ better** |
| P+ vs H3 | write_p99_us | 3408 | 317.7 | 5193 | 1048 | 774.3 | 2436 | +52.4% | 2.31 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 744.9 | 36.9 | 525.2 | 72.3 | 57.4 | 95.5 | -29.5% | 3.83 | **P+ better** |
| P+ vs H3 | pss_mib | 47.2 | 3.97 | 107.6 | 1.98 | 3.14 | 0.17 | +128.0% | 19.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 465.6 | 1790 | 19632 | 3788 | 2963 | 41712 | +4116.8% | 6.47 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 194.7 | 12.7 | 1442 | 68.3 | 49.1 | 248.3 | +640.6% | 25.4 | **N better** |
| N vs T | q1_p99_us | 657.8 | 156.0 | 4985 | 2232 | 1582 | 969.3 | +657.8% | 2.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q2_p50_us | 77307 | 6735 | 1406 | 79.9 | 4763 | 8410 | -98.2% | 15.9 | **T better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 159.6 | 16.9 | 1901 | 114.9 | 82.1 | 195.6 | +1091.4% | 21.2 | **N better** |
| N vs T | c2_q1_p99_us | 2822 | 1260 | 6256 | 1551 | 1413 | 540.1 | +121.7% | 2.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q2_p50_us | 88571 | 6779 | 1918 | 139.7 | 4795 | 5913 | -97.8% | 18.1 | **T better** |
| N vs T | c4_q1_p50_us | 181.7 | 26.3 | 2600 | 246.6 | 175.3 | 149.5 | +1330.5% | 13.8 | **N better** |
| N vs T | c4_q1_p99_us | 7563 | 599.9 | 7977 | 2792 | 2019 | 2151 | +5.5% | 0.21 | BELOW FLOOR |
| N vs T | c4_q2_p50_us | 119671 | 4501 | 2686 | 213.7 | 3186 | 11734 | -97.8% | 36.7 | **T better** |
| N vs T | c4_q2_p99_us | 412769 | 23267 | 9526 | 2899 | 16580 | 55161 | -97.7% | 24.3 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 622.4 | 39.5 | 1298 | 61.5 | 51.7 | 198.1 | +108.6% | 13.1 | **N better** |
| N vs T | write_p99_us | 1999 | 936.7 | 4428 | 2463 | 1864 | 2052 | +121.5% | 1.30 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | commits_per_s | 1299 | 237.7 | 633.5 | 93.0 | 180.5 | 426.0 | -51.2% | 3.69 | **N better** |
| N vs T | pss_mib | 178.5 | 40.5 | 2580 | 0.15 | 28.7 | 0.56 | +1345.9% | 83.8 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 50381 | 4218 | -704.4 | 202.0 | 2986 | 69014 | -101.4% | 17.1 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 747.6 | 55.8 | 1442 | 68.3 | 62.4 | 248.2 | +92.9% | 11.1 | **P+ better** |
| P+ vs T | q1_p99_us | 2477 | 817.6 | 4985 | 2232 | 1681 | 1365 | +101.2% | 1.49 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | q2_p50_us | 525.3 | 29.2 | 1406 | 79.9 | 60.2 | 124.7 | +167.7% | 14.6 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 938.8 | 105.4 | 1901 | 114.9 | 110.2 | 195.7 | +102.5% | 8.73 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 5541 | 2197 | 6256 | 1551 | 1901 | 1328 | +12.9% | 0.38 | BELOW FLOOR |
| P+ vs T | c2_q2_p50_us | 708.4 | 41.8 | 1918 | 139.7 | 103.1 | 40.0 | +170.7% | 11.7 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2032 | 124.8 | 2600 | 246.6 | 195.4 | 149.3 | +27.9% | 2.91 | no difference |
| P+ vs T | c4_q1_p99_us | 11372 | 3262 | 7977 | 2792 | 3036 | 2433 | -29.9% | 1.12 | no difference |
| P+ vs T | c4_q2_p50_us | 1828 | 126.2 | 2686 | 213.7 | 175.5 | 652.3 | +46.9% | 4.89 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 8945 | 3695 | 9526 | 2899 | 3321 | 10520 | +6.5% | 0.18 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | write_p50_us | 1117 | 107.1 | 1298 | 61.5 | 87.4 | 205.1 | +16.2% | 2.07 | BELOW FLOOR |
| P+ vs T | write_p99_us | 3408 | 317.7 | 4428 | 2463 | 1756 | 2757 | +29.9% | 0.58 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | commits_per_s | 744.9 | 36.9 | 633.5 | 93.0 | 70.7 | 229.6 | -15.0% | 1.58 | BELOW FLOOR |
| P+ vs T | pss_mib | 47.2 | 3.97 | 2580 | 0.15 | 2.81 | 0.21 | +5366.5% | 902.1 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | 465.6 | 1790 | -704.4 | 202.0 | 1274 | 2917 | -251.3% | 0.92 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 470.4 | 38.6 | 1442 | 68.3 | 55.4 | 249.5 | +206.5% | 17.5 | **H3 better** |
| H3 vs T | q1_p99_us | 1586 | 351.7 | 4985 | 2232 | 1598 | 848.5 | +214.3% | 2.13 | no difference |
| H3 vs T | q2_p50_us | 444.6 | 75.3 | 1406 | 79.9 | 77.6 | 84.8 | +216.2% | 12.4 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 399.6 | 37.3 | 1901 | 114.9 | 85.4 | 267.4 | +375.8% | 17.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q1_p99_us | 1706 | 515.8 | 6256 | 1551 | 1156 | 2852 | +266.6% | 3.94 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q2_p50_us | 397.1 | 37.2 | 1918 | 139.7 | 102.2 | 160.2 | +382.9% | 14.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p50_us | 733.7 | 44.0 | 2600 | 246.6 | 177.1 | 159.7 | +254.3% | 10.5 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 2834 | 642.3 | 7977 | 2792 | 2026 | 1786 | +181.5% | 2.54 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p50_us | 769.3 | 49.6 | 2686 | 213.7 | 155.1 | 581.3 | +249.1% | 12.4 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 2608 | 444.8 | 9526 | 2899 | 2074 | 10874 | +265.3% | 3.34 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | write_p50_us | 1569 | 92.2 | 1298 | 61.5 | 78.4 | 234.0 | -17.3% | 3.46 | **T better** |
| H3 vs T | write_p99_us | 5193 | 1048 | 4428 | 2463 | 1893 | 1557 | -14.7% | 0.40 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 525.2 | 72.3 | 633.5 | 93.0 | 83.3 | 247.2 | +20.6% | 1.30 | BELOW FLOOR |
| H3 vs T | pss_mib | 107.6 | 1.98 | 2580 | 0.15 | 1.40 | 0.23 | +2297.0% | 1762 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 19632 | 3788 | -704.4 | 202.0 | 2683 | 41723 | -103.6% | 7.58 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 194.7 | 12.7 | 1138 | 85.5 | 61.1 | 365.3 | +484.7% | 15.4 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q1_p99_us | 657.8 | 156.0 | 7151 | 2345 | 1662 | 1029 | +987.1% | 3.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p50_us | 77307 | 6735 | 902.7 | 68.1 | 4763 | 8410 | -98.8% | 16.0 | **H1 better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 12777 | 3996 | 41415 | 5727 | 4938 | 18298 | +224.1% | 5.80 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | q6_p50_us | 59721 | 1892 | 11619 | 591.8 | 1402 | 1631 | -80.5% | 34.3 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 159.6 | 16.9 | 983.5 | 105.3 | 75.4 | 109.5 | +516.3% | 10.9 | **N better** |
| N vs H1 | c2_q1_p99_us | 2822 | 1260 | 6972 | 1731 | 1514 | 2337 | +147.1% | 2.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q2_p50_us | 88571 | 6779 | 1046 | 87.9 | 4794 | 5913 | -98.8% | 18.3 | **H1 better** |
| N vs H1 | c4_q1_p50_us | 181.7 | 26.3 | 1046 | 128.5 | 92.8 | 50.7 | +475.6% | 9.32 | **N better** |
| N vs H1 | c4_q1_p99_us | 7563 | 599.9 | 5316 | 645.7 | 623.2 | 2125 | -29.7% | 3.61 | **H1 better** |
| N vs H1 | c4_q2_p50_us | 119671 | 4501 | 972.1 | 121.1 | 3184 | 11722 | -99.2% | 37.3 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 412769 | 23267 | 4426 | 503.0 | 16456 | 54767 | -98.9% | 24.8 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | write_p50_us | 622.4 | 39.5 | 1769 | 100.7 | 76.5 | 38.2 | +184.1% | 15.0 | **N better** |
| N vs H1 | write_p99_us | 1999 | 936.7 | 5826 | 647.1 | 805.1 | 1533 | +191.4% | 4.75 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | commits_per_s | 1299 | 237.7 | 446.7 | 10.5 | 168.2 | 361.2 | -65.6% | 5.07 | **N better** |
| N vs H1 | pss_mib | 178.5 | 40.5 | 420.2 | 6.40 | 29.0 | 3.29 | +135.5% | 8.33 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 50381 | 4218 | 24429 | 2827 | 3590 | 72296 | -51.5% | 7.23 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| P+ vs H1 | q1_p50_us | 747.6 | 55.8 | 1138 | 85.5 | 72.2 | 365.2 | +52.3% | 5.41 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q1_p99_us | 2477 | 817.6 | 7151 | 2345 | 1756 | 1408 | +188.7% | 2.66 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q2_p50_us | 525.3 | 29.2 | 902.7 | 68.1 | 52.4 | 135.6 | +71.8% | 7.20 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 314.8 | 15.3 | 982.4 | 97.9 | 70.1 | 236.7 | +212.1% | 9.53 | **P+ better** |
| P+ vs H1 | q4_p50_us | 16410 | 1216 | 31051 | 3554 | 2656 | 2100 | +89.2% | 5.51 | **P+ better** |
| P+ vs H1 | q5_p50_us | 29924 | 10911 | 41415 | 5727 | 8714 | 16551 | +38.4% | 1.32 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q6_p50_us | 60986 | 4664 | 11619 | 591.8 | 3324 | 7084 | -80.9% | 14.9 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 938.8 | 105.4 | 983.5 | 105.3 | 105.3 | 109.6 | +4.8% | 0.42 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 5541 | 2197 | 6972 | 1731 | 1978 | 2633 | +25.8% | 0.72 | BELOW FLOOR |
| P+ vs H1 | c2_q2_p50_us | 708.4 | 41.8 | 1046 | 87.9 | 68.8 | 57.1 | +47.7% | 4.91 | **P+ better** |
| P+ vs H1 | c4_q1_p50_us | 2032 | 124.8 | 1046 | 128.5 | 126.7 | 49.8 | -48.5% | 7.78 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 11372 | 3262 | 5316 | 645.7 | 2351 | 2409 | -53.3% | 2.58 | no difference |
| P+ vs H1 | c4_q2_p50_us | 1828 | 126.2 | 972.1 | 121.1 | 123.7 | 396.0 | -46.8% | 6.92 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 8945 | 3695 | 4426 | 503.0 | 2637 | 8208 | -50.5% | 1.71 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | write_p50_us | 1117 | 107.1 | 1769 | 100.7 | 104.0 | 65.3 | +58.3% | 6.26 | **P+ better** |
| P+ vs H1 | write_p99_us | 3408 | 317.7 | 5826 | 647.1 | 509.8 | 2396 | +70.9% | 4.74 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | commits_per_s | 744.9 | 36.9 | 446.7 | 10.5 | 27.1 | 40.7 | -40.0% | 11.0 | **P+ better** |
| P+ vs H1 | pss_mib | 47.2 | 3.97 | 420.2 | 6.40 | 5.33 | 3.25 | +790.3% | 70.0 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | 465.6 | 1790 | 24429 | 2827 | 2366 | 21733 | +5147.0% | 10.1 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 194.7 | 12.7 | 145.0 | 14.6 | 13.7 | 39.4 | -25.5% | 3.63 | **H2 better** |
| N vs H2 | q1_p99_us | 657.8 | 156.0 | 762.4 | 291.2 | 233.6 | 489.9 | +15.9% | 0.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 77307 | 6735 | 239.1 | 34.0 | 4763 | 8410 | -99.7% | 16.2 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 12777 | 3996 | 226.7 | 27.3 | 2826 | 8593 | -98.2% | 4.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q6_p50_us | 59721 | 1892 | 6108 | 960.1 | 1500 | 2083 | -89.8% | 35.7 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 159.6 | 16.9 | 132.4 | 27.0 | 22.5 | 10.0 | -17.0% | 1.21 | no difference |
| N vs H2 | c2_q1_p99_us | 2822 | 1260 | 1514 | 269.8 | 911.2 | 2623 | -46.4% | 1.44 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 88571 | 6779 | 264.7 | 45.3 | 4794 | 5913 | -99.7% | 18.4 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 181.7 | 26.3 | 207.3 | 23.1 | 24.8 | 33.4 | +14.1% | 1.03 | BELOW FLOOR |
| N vs H2 | c4_q1_p99_us | 7563 | 599.9 | 3168 | 542.5 | 571.9 | 2132 | -58.1% | 7.68 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 119671 | 4501 | 394.1 | 43.0 | 3183 | 11720 | -99.7% | 37.5 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 412769 | 23267 | 4702 | 2009 | 16514 | 54254 | -98.9% | 24.7 | **H2 better** |
| N vs H2 | write_p50_us | 622.4 | 39.5 | 1689 | 249.7 | 178.8 | 118.2 | +171.3% | 5.96 | **N better** |
| N vs H2 | write_p99_us | 1999 | 936.7 | 5748 | 902.9 | 919.9 | 1472 | +187.5% | 4.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1299 | 237.7 | 486.4 | 49.7 | 171.7 | 359.6 | -62.6% | 4.73 | **N better** |
| N vs H2 | pss_mib | 178.5 | 40.5 | 43.4 | 0.06 | 28.7 | 0.53 | -75.7% | 4.71 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 50381 | 4218 | -789.2 | 559.6 | 3008 | 69006 | -101.6% | 17.0 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 747.6 | 55.8 | 145.0 | 14.6 | 40.8 | 38.6 | -80.6% | 14.8 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2477 | 817.6 | 762.4 | 291.2 | 613.7 | 1079 | -69.2% | 2.79 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 525.3 | 29.2 | 239.1 | 34.0 | 31.7 | 92.4 | -54.5% | 9.03 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 314.8 | 15.3 | 333.8 | 63.4 | 46.1 | 162.6 | +6.0% | 0.41 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q4_p50_us | 16410 | 1216 | 1291 | 77.9 | 861.8 | 590.3 | -92.1% | 17.5 | **H2 better** |
| P+ vs H2 | q5_p50_us | 29924 | 10911 | 226.7 | 27.3 | 7715 | 3600 | -99.2% | 3.85 | **H2 better** |
| P+ vs H2 | q6_p50_us | 60986 | 4664 | 6108 | 960.1 | 3367 | 7202 | -90.0% | 16.3 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 938.8 | 105.4 | 132.4 | 27.0 | 76.9 | 10.6 | -85.9% | 10.5 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5541 | 2197 | 1514 | 269.8 | 1565 | 2890 | -72.7% | 2.57 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q2_p50_us | 708.4 | 41.8 | 264.7 | 45.3 | 43.6 | 48.6 | -62.6% | 10.2 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 2032 | 124.8 | 207.3 | 23.1 | 89.8 | 32.1 | -89.8% | 20.3 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 11372 | 3262 | 3168 | 542.5 | 2338 | 2416 | -72.1% | 3.51 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1828 | 126.2 | 394.1 | 43.0 | 94.3 | 311.7 | -78.4% | 15.2 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 8945 | 3695 | 4702 | 2009 | 2974 | 3385 | -47.4% | 1.43 | no difference |
| P+ vs H2 | write_p50_us | 1117 | 107.1 | 1689 | 249.7 | 192.1 | 129.6 | +51.1% | 2.97 | no difference |
| P+ vs H2 | write_p99_us | 3408 | 317.7 | 5748 | 902.9 | 676.8 | 2358 | +68.6% | 3.46 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 744.9 | 36.9 | 486.4 | 49.7 | 43.8 | 23.1 | -34.7% | 5.90 | **P+ better** |
| P+ vs H2 | pss_mib | 47.2 | 3.97 | 43.4 | 0.06 | 2.81 | 0.10 | -8.0% | 1.35 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | 465.6 | 1790 | -789.2 | 559.6 | 1326 | 2738 | -269.5% | 0.95 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 181.1 | 18.2 | 688.3 | 51.8 | 38.8 | 111.5 | +280.0% | 13.1 | **N better** |
| N vs P+ | q1_p99_us | 674.6 | 97.8 | 2460 | 987.7 | 701.8 | 161.2 | +264.7% | 2.54 | no difference |
| N vs P+ | q2_p50_us | 71426 | 4400 | 545.1 | 14.9 | 3111 | 6268 | -99.2% | 22.8 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 28108 | 4501 | 43823 | 9153 | 7213 | 9727 | +55.9% | 2.18 | no difference |
| N vs P+ | q6_p50_us | 55681 | 1456 | 63352 | 2633 | 2128 | 5502 | +13.8% | 3.61 | **N better** |
| N vs P+ | c2_q1_p50_us | 165.1 | 28.0 | 957.8 | 175.1 | 125.4 | 218.5 | +480.2% | 6.32 | **N better** |
| N vs P+ | c2_q1_p99_us | 3264 | 389.5 | 6880 | 1213 | 901.0 | 8475 | +110.8% | 4.01 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 94520 | 9570 | 759.5 | 88.2 | 6767 | 13739 | -99.2% | 13.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 203.9 | 29.5 | 1949 | 147.5 | 106.4 | 313.4 | +855.8% | 16.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 7618 | 629.7 | 9767 | 1295 | 1019 | 1842 | +28.2% | 2.11 | no difference |
| N vs P+ | c4_q2_p50_us | 116399 | 7370 | 1822 | 97.7 | 5212 | 12639 | -98.4% | 22.0 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 387609 | 15837 | 11107 | 2496 | 11337 | 21672 | -97.1% | 33.2 | **P+ better** |
| N vs P+ | write_p50_us | 553.9 | 12.2 | 994.1 | 92.1 | 65.7 | 162.7 | +79.5% | 6.70 | **N better** |
| N vs P+ | write_p99_us | 1998 | 928.5 | 3331 | 200.1 | 671.6 | 460.0 | +66.8% | 1.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1502 | 156.5 | 814.9 | 73.3 | 122.2 | 356.9 | -45.7% | 5.62 | **N better** |
| N vs P+ | pss_mib | 187.3 | 25.0 | 44.4 | 1.17 | 17.7 | 25.5 | -76.3% | 8.07 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 57129 | 4792 | -469.5 | 950.2 | 3454 | 10814 | -100.8% | 16.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 181.1 | 18.2 | 97.0 | 24.5 | 21.6 | 23.0 | -46.5% | 3.90 | **P better** |
| N vs P | q1_p99_us | 674.6 | 97.8 | 621.2 | 168.0 | 137.4 | 138.3 | -7.9% | 0.39 | BELOW FLOOR |
| N vs P | q2_p50_us | 71426 | 4400 | 203.6 | 18.6 | 3111 | 6268 | -99.7% | 22.9 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 28108 | 4501 | 154.2 | 14.3 | 3182 | 2703 | -99.5% | 8.78 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 55681 | 1456 | 6065 | 270.3 | 1047 | 207.0 | -89.1% | 47.4 | **P better** |
| N vs P | c2_q1_p50_us | 165.1 | 28.0 | 123.4 | 20.7 | 24.6 | 30.1 | -25.2% | 1.69 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 3264 | 389.5 | 2591 | 540.6 | 471.1 | 2665 | -20.6% | 1.43 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 94520 | 9570 | 284.8 | 23.6 | 6767 | 13735 | -99.7% | 13.9 | **P better** |
| N vs P | c4_q1_p50_us | 203.9 | 29.5 | 175.4 | 27.9 | 28.7 | 81.0 | -14.0% | 0.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 7618 | 629.7 | 3585 | 478.8 | 559.4 | 1585 | -52.9% | 7.21 | **P better** |
| N vs P | c4_q2_p50_us | 116399 | 7370 | 381.1 | 52.2 | 5212 | 12635 | -99.7% | 22.3 | **P better** |
| N vs P | c4_q2_p99_us | 387609 | 15837 | 5898 | 1896 | 11279 | 21707 | -98.5% | 33.8 | **P better** |
| N vs P | write_p50_us | 553.9 | 12.2 | 193933 | 4560 | 3224 | 4968 | +34914.9% | 60.0 | **N better** |
| N vs P | write_p99_us | 1998 | 928.5 | 252308 | 12204 | 8655 | 6463 | +12530.1% | 28.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1502 | 156.5 | 5.00 | 0.17 | 110.6 | 352.5 | -99.7% | 13.5 | **N better** |
| N vs P | pss_mib | 187.3 | 25.0 | 43.4 | 0.07 | 17.7 | 25.5 | -76.8% | 8.13 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 57129 | 4792 | -700.3 | 477.8 | 3405 | 10736 | -101.2% | 17.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 688.3 | 51.8 | 97.0 | 24.5 | 40.5 | 109.2 | -85.9% | 14.6 | **P better** |
| P+ vs P | q1_p99_us | 2460 | 987.7 | 621.2 | 168.0 | 708.4 | 180.7 | -74.7% | 2.60 | no difference |
| P+ vs P | q2_p50_us | 545.1 | 14.9 | 203.6 | 18.6 | 16.9 | 37.6 | -62.7% | 20.2 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 327.4 | 46.4 | 198.1 | 36.6 | 41.8 | 54.1 | -39.5% | 3.09 | **P better** |
| P+ vs P | q4_p50_us | 16100 | 943.1 | 1036 | 52.5 | 667.9 | 703.2 | -93.6% | 22.6 | **P better** |
| P+ vs P | q5_p50_us | 43823 | 9153 | 154.2 | 14.3 | 6473 | 9344 | -99.6% | 6.75 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 63352 | 2633 | 6065 | 270.3 | 1872 | 5500 | -90.4% | 30.6 | **P better** |
| P+ vs P | c2_q1_p50_us | 957.8 | 175.1 | 123.4 | 20.7 | 124.7 | 219.0 | -87.1% | 6.69 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 6880 | 1213 | 2591 | 540.6 | 939.2 | 8159 | -62.3% | 4.57 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 759.5 | 88.2 | 284.8 | 23.6 | 64.6 | 320.6 | -62.5% | 7.35 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p50_us | 1949 | 147.5 | 175.4 | 27.9 | 106.1 | 304.9 | -91.0% | 16.7 | **P better** |
| P+ vs P | c4_q1_p99_us | 9767 | 1295 | 3585 | 478.8 | 976.6 | 976.6 | -63.3% | 6.33 | **P better** |
| P+ vs P | c4_q2_p50_us | 1822 | 97.7 | 381.1 | 52.2 | 78.3 | 317.7 | -79.1% | 18.4 | **P better** |
| P+ vs P | c4_q2_p99_us | 11107 | 2496 | 5898 | 1896 | 2216 | 1279 | -46.9% | 2.35 | no difference |
| P+ vs P | write_p50_us | 994.1 | 92.1 | 193933 | 4560 | 3225 | 4968 | +19408.3% | 59.8 | **P+ better** |
| P+ vs P | write_p99_us | 3331 | 200.1 | 252308 | 12204 | 8631 | 6454 | +7473.8% | 28.8 | **P+ better** |
| P+ vs P | commits_per_s | 814.9 | 73.3 | 5.00 | 0.17 | 51.8 | 56.2 | -99.4% | 15.6 | **P+ better** |
| P+ vs P | pss_mib | 44.4 | 1.17 | 43.4 | 0.07 | 0.83 | 0.23 | -2.2% | 1.19 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -469.5 | 950.2 | -700.3 | 477.8 | 752.1 | 2697 | +49.2% | 0.31 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 97.0 | 24.5 | 103.1 | 20.7 | 22.7 | 3.09 | +6.3% | 0.27 | no difference |
| P vs M | q1_p99_us | 621.2 | 168.0 | 534.2 | 157.4 | 162.8 | 226.2 | -14.0% | 0.53 | BELOW FLOOR |
| P vs M | q2_p50_us | 203.6 | 18.6 | 197.8 | 15.5 | 17.2 | 7.79 | -2.8% | 0.34 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 198.1 | 36.6 | 196.9 | 22.0 | 30.2 | 21.9 | -0.6% | 0.04 | BELOW FLOOR |
| P vs M | q4_p50_us | 1036 | 52.5 | 1020 | 31.9 | 43.5 | 293.4 | -1.5% | 0.37 | BELOW FLOOR |
| P vs M | q5_p50_us | 154.2 | 14.3 | 146.5 | 19.7 | 17.2 | 89.4 | -5.0% | 0.45 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 6065 | 270.3 | 6059 | 671.8 | 512.1 | 649.8 | -0.1% | 0.01 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 123.4 | 20.7 | 128.1 | 43.3 | 34.0 | 31.9 | +3.8% | 0.14 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 2591 | 540.6 | 1884 | 409.8 | 479.7 | 1052 | -27.3% | 1.48 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 284.8 | 23.6 | 264.9 | 72.4 | 53.8 | 57.0 | -7.0% | 0.37 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 175.4 | 27.9 | 197.6 | 19.0 | 23.8 | 31.4 | +12.7% | 0.93 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 3585 | 478.8 | 2653 | 439.7 | 459.7 | 442.3 | -26.0% | 2.03 | no difference |
| P vs M | c4_q2_p50_us | 381.1 | 52.2 | 356.1 | 44.0 | 48.3 | 89.7 | -6.5% | 0.52 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 5898 | 1896 | 4123 | 511.5 | 1389 | 2145 | -30.1% | 1.28 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 193933 | 4560 | 194867 | 2986 | 3854 | 8222 | +0.5% | 0.24 | BELOW FLOOR |
| P vs M | write_p99_us | 252308 | 12204 | 245175 | 7874 | 10270 | 14211 | -2.8% | 0.69 | BELOW FLOOR |
| P vs M | commits_per_s | 5.00 | 0.17 | 5.00 | 0.06 | 0.13 | 0.26 | +0.1% | 0.04 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.07 | 43.4 | 0.08 | 0.08 | 0.12 | +0.0% | 0.05 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -700.3 | 477.8 | -730.6 | 475.2 | 476.5 | 2542 | +4.3% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 181.1 | 18.2 | 439.8 | 25.4 | 22.1 | 34.3 | +142.8% | 11.7 | **N better** |
| N vs H3 | q1_p99_us | 674.6 | 97.8 | 1837 | 520.7 | 374.6 | 1527 | +172.3% | 3.10 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 71426 | 4400 | 454.0 | 13.8 | 3111 | 6268 | -99.4% | 22.8 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 28108 | 4501 | 44310 | 8263 | 6653 | 5266 | +57.6% | 2.44 | no difference |
| N vs H3 | q6_p50_us | 55681 | 1456 | 62320 | 4642 | 3440 | 6380 | +11.9% | 1.93 | no difference |
| N vs H3 | c2_q1_p50_us | 165.1 | 28.0 | 481.3 | 84.5 | 62.9 | 32.8 | +191.6% | 5.03 | **N better** |
| N vs H3 | c2_q1_p99_us | 3264 | 389.5 | 2289 | 720.4 | 579.1 | 3320 | -29.9% | 1.68 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 94520 | 9570 | 448.0 | 65.2 | 6767 | 13735 | -99.5% | 13.9 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 203.9 | 29.5 | 715.3 | 100.3 | 73.9 | 118.0 | +250.9% | 6.92 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 7618 | 629.7 | 3089 | 320.6 | 499.7 | 1754 | -59.5% | 9.06 | **H3 better** |
| N vs H3 | c4_q2_p50_us | 116399 | 7370 | 743.2 | 110.6 | 5212 | 12636 | -99.4% | 22.2 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 387609 | 15837 | 3305 | 640.7 | 11208 | 21673 | -99.1% | 34.3 | **H3 better** |
| N vs H3 | write_p50_us | 553.9 | 12.2 | 1433 | 81.0 | 57.9 | 163.4 | +158.8% | 15.2 | **N better** |
| N vs H3 | write_p99_us | 1998 | 928.5 | 9808 | 5508 | 3950 | 1439 | +391.0% | 1.98 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1502 | 156.5 | 500.0 | 86.7 | 126.5 | 352.5 | -66.7% | 7.92 | **N better** |
| N vs H3 | pss_mib | 187.3 | 25.0 | 109.6 | 0.08 | 17.7 | 25.5 | -41.5% | 4.39 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 57129 | 4792 | 20050 | 3472 | 4184 | 31219 | -64.9% | 8.86 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 688.3 | 51.8 | 439.8 | 25.4 | 40.8 | 112.1 | -36.1% | 6.09 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2460 | 987.7 | 1837 | 520.7 | 789.5 | 1531 | -25.3% | 0.79 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 545.1 | 14.9 | 454.0 | 13.8 | 14.4 | 37.8 | -16.7% | 6.34 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 327.4 | 46.4 | 427.6 | 92.0 | 72.8 | 55.2 | +30.6% | 1.38 | no difference |
| P+ vs H3 | q4_p50_us | 16100 | 943.1 | 15487 | 543.9 | 769.8 | 1082 | -3.8% | 0.80 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 43823 | 9153 | 44310 | 8263 | 8719 | 10380 | +1.1% | 0.06 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 63352 | 2633 | 62320 | 4642 | 3774 | 8421 | -1.6% | 0.27 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 957.8 | 175.1 | 481.3 | 84.5 | 137.5 | 219.4 | -49.7% | 3.46 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 6880 | 1213 | 2289 | 720.4 | 997.7 | 8395 | -66.7% | 4.60 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 759.5 | 88.2 | 448.0 | 65.2 | 77.6 | 320.6 | -41.0% | 4.01 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 1949 | 147.5 | 715.3 | 100.3 | 126.1 | 316.7 | -63.3% | 9.78 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 9767 | 1295 | 3089 | 320.6 | 943.7 | 1232 | -68.4% | 7.08 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1822 | 97.7 | 743.2 | 110.6 | 104.3 | 354.7 | -59.2% | 10.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 11107 | 2496 | 3305 | 640.7 | 1822 | 389.4 | -70.2% | 4.28 | **H3 better** |
| P+ vs H3 | write_p50_us | 994.1 | 92.1 | 1433 | 81.0 | 86.7 | 179.7 | +44.2% | 5.06 | **P+ better** |
| P+ vs H3 | write_p99_us | 3331 | 200.1 | 9808 | 5508 | 3898 | 1398 | +194.4% | 1.66 | no difference |
| P+ vs H3 | commits_per_s | 814.9 | 73.3 | 500.0 | 86.7 | 80.3 | 56.2 | -38.6% | 3.92 | **P+ better** |
| P+ vs H3 | pss_mib | 44.4 | 1.17 | 109.6 | 0.08 | 0.83 | 0.37 | +146.7% | 78.8 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -469.5 | 950.2 | 20050 | 3472 | 2545 | 29439 | -4370.7% | 8.06 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 181.1 | 18.2 | 1412 | 92.3 | 66.5 | 97.0 | +679.4% | 18.5 | **N better** |
| N vs T | q1_p99_us | 674.6 | 97.8 | 4750 | 1897 | 1343 | 5982 | +604.2% | 3.03 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | q2_p50_us | 71426 | 4400 | 1430 | 108.0 | 3112 | 6279 | -98.0% | 22.5 | **T better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 165.1 | 28.0 | 1986 | 101.3 | 74.3 | 18.7 | +1103.1% | 24.5 | **N better** |
| N vs T | c2_q1_p99_us | 3264 | 389.5 | 5376 | 2052 | 1477 | 5375 | +64.7% | 1.43 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c2_q2_p50_us | 94520 | 9570 | 2011 | 54.4 | 6767 | 13736 | -97.9% | 13.7 | **T better** |
| N vs T | c4_q1_p50_us | 203.9 | 29.5 | 2459 | 75.8 | 57.5 | 549.3 | +1106.0% | 39.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 7618 | 629.7 | 7141 | 1599 | 1215 | 3056 | -6.3% | 0.39 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c4_q2_p50_us | 116399 | 7370 | 2574 | 115.4 | 5212 | 12639 | -97.8% | 21.8 | **T better** |
| N vs T | c4_q2_p99_us | 387609 | 15837 | 5757 | 1503 | 11249 | 21800 | -98.5% | 33.9 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 553.9 | 12.2 | 1327 | 42.2 | 31.0 | 245.1 | +139.5% | 24.9 | **N better** |
| N vs T | write_p99_us | 1998 | 928.5 | 3004 | 1138 | 1038 | 1647 | +50.4% | 0.97 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | commits_per_s | 1502 | 156.5 | 690.8 | 75.9 | 123.0 | 352.7 | -54.0% | 6.59 | **N better** |
| N vs T | pss_mib | 187.3 | 25.0 | 2581 | 0.31 | 17.7 | 25.5 | +1278.4% | 135.3 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 57129 | 4792 | 145.0 | 60.5 | 3389 | 10605 | -99.7% | 16.8 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | q1_p50_us | 688.3 | 51.8 | 1412 | 92.3 | 74.8 | 144.2 | +105.1% | 9.66 | **P+ better** |
| P+ vs T | q1_p99_us | 2460 | 987.7 | 4750 | 1897 | 1513 | 5983 | +93.1% | 1.51 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | q2_p50_us | 545.1 | 14.9 | 1430 | 108.0 | 77.1 | 363.7 | +162.3% | 11.5 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 957.8 | 175.1 | 1986 | 101.3 | 143.1 | 217.8 | +107.4% | 7.19 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 6880 | 1213 | 5376 | 2052 | 1685 | 9400 | -21.9% | 0.89 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c2_q2_p50_us | 759.5 | 88.2 | 2011 | 54.4 | 73.3 | 347.4 | +164.7% | 17.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q1_p50_us | 1949 | 147.5 | 2459 | 75.8 | 117.3 | 623.0 | +26.2% | 4.35 | BELOW FLOOR |
| P+ vs T | c4_q1_p99_us | 9767 | 1295 | 7141 | 1599 | 1455 | 2790 | -26.9% | 1.80 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c4_q2_p50_us | 1822 | 97.7 | 2574 | 115.4 | 106.9 | 438.3 | +41.2% | 7.03 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 11107 | 2496 | 5757 | 1503 | 2060 | 2379 | -48.2% | 2.60 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | write_p50_us | 994.1 | 92.1 | 1327 | 42.2 | 71.7 | 256.2 | +33.5% | 4.64 | **P+ better** |
| P+ vs T | write_p99_us | 3331 | 200.1 | 3004 | 1138 | 816.8 | 1612 | -9.8% | 0.40 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 814.9 | 73.3 | 690.8 | 75.9 | 74.6 | 58.0 | -15.2% | 1.66 | no difference |
| P+ vs T | pss_mib | 44.4 | 1.17 | 2581 | 0.31 | 0.85 | 0.22 | +5711.9% | 2972 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -469.5 | 950.2 | 145.0 | 60.5 | 673.3 | 2116 | -130.9% | 0.91 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 439.8 | 25.4 | 1412 | 92.3 | 67.7 | 97.7 | +221.0% | 14.4 | **H3 better** |
| H3 vs T | q1_p99_us | 1837 | 520.7 | 4750 | 1897 | 1391 | 6173 | +158.6% | 2.09 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | q2_p50_us | 454.0 | 13.8 | 1430 | 108.0 | 77.0 | 361.8 | +214.9% | 12.7 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 481.3 | 84.5 | 1986 | 101.3 | 93.3 | 27.7 | +312.7% | 16.1 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 2289 | 720.4 | 5376 | 2052 | 1537 | 5249 | +134.8% | 2.01 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c2_q2_p50_us | 448.0 | 65.2 | 2011 | 54.4 | 60.1 | 133.9 | +348.8% | 26.0 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 715.3 | 100.3 | 2459 | 75.8 | 88.9 | 551.2 | +243.7% | 19.6 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 3089 | 320.6 | 7141 | 1599 | 1153 | 2733 | +131.2% | 3.51 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c4_q2_p50_us | 743.2 | 110.6 | 2574 | 115.4 | 113.0 | 362.9 | +246.3% | 16.2 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 3305 | 640.7 | 5757 | 1503 | 1156 | 2387 | +74.2% | 2.12 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | write_p50_us | 1433 | 81.0 | 1327 | 42.2 | 64.6 | 256.7 | -7.4% | 1.65 | BELOW FLOOR |
| H3 vs T | write_p99_us | 9808 | 5508 | 3004 | 1138 | 3977 | 2111 | -69.4% | 1.71 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 500.0 | 86.7 | 690.8 | 75.9 | 81.5 | 14.3 | +38.2% | 2.34 | no difference |
| H3 vs T | pss_mib | 109.6 | 0.08 | 2581 | 0.31 | 0.23 | 0.30 | +2255.5% | 10906 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 20050 | 3472 | 145.0 | 60.5 | 2455 | 29363 | -99.3% | 8.11 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 181.1 | 18.2 | 1025 | 77.4 | 56.2 | 154.3 | +465.7% | 15.0 | **N better** |
| N vs H1 | q1_p99_us | 674.6 | 97.8 | 5701 | 1008 | 715.9 | 887.1 | +745.2% | 7.02 | **N better** |
| N vs H1 | q2_p50_us | 71426 | 4400 | 888.7 | 73.7 | 3112 | 6269 | -98.8% | 22.7 | **H1 better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 28108 | 4501 | 48299 | 47706 | 33883 | 2838 | +71.8% | 0.60 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q6_p50_us | 55681 | 1456 | 12232 | 1712 | 1589 | 219.3 | -78.0% | 27.3 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 165.1 | 28.0 | 970.4 | 150.8 | 108.4 | 83.6 | +487.8% | 7.43 | **N better** |
| N vs H1 | c2_q1_p99_us | 3264 | 389.5 | 5316 | 1390 | 1021 | 3556 | +62.9% | 2.01 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | c2_q2_p50_us | 94520 | 9570 | 952.7 | 92.9 | 6767 | 13735 | -99.0% | 13.8 | **H1 better** |
| N vs H1 | c4_q1_p50_us | 203.9 | 29.5 | 981.8 | 81.2 | 61.1 | 115.6 | +381.5% | 12.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 7618 | 629.7 | 5473 | 1129 | 914.0 | 2104 | -28.2% | 2.35 | no difference |
| N vs H1 | c4_q2_p50_us | 116399 | 7370 | 958.5 | 67.6 | 5212 | 12637 | -99.2% | 22.1 | **H1 better** |
| N vs H1 | c4_q2_p99_us | 387609 | 15837 | 5027 | 1407 | 11243 | 21672 | -98.7% | 34.0 | **H1 better** |
| N vs H1 | write_p50_us | 553.9 | 12.2 | 1652 | 112.7 | 80.1 | 191.8 | +198.3% | 13.7 | **N better** |
| N vs H1 | write_p99_us | 1998 | 928.5 | 5415 | 675.0 | 811.7 | 946.5 | +171.1% | 4.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | commits_per_s | 1502 | 156.5 | 502.9 | 45.3 | 115.2 | 353.4 | -66.5% | 8.67 | **N better** |
| N vs H1 | pss_mib | 187.3 | 25.0 | 422.3 | 9.08 | 18.8 | 25.6 | +125.5% | 12.5 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 57129 | 4792 | 15990 | 1929 | 3653 | 27070 | -72.0% | 11.3 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q1_p50_us | 688.3 | 51.8 | 1025 | 77.4 | 65.8 | 187.6 | +48.9% | 5.11 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2460 | 987.7 | 5701 | 1008 | 997.7 | 894.7 | +131.8% | 3.25 | **P+ better** |
| P+ vs H1 | q2_p50_us | 545.1 | 14.9 | 888.7 | 73.7 | 53.1 | 112.5 | +63.0% | 6.47 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 327.4 | 46.4 | 873.1 | 160.5 | 118.2 | 139.5 | +166.7% | 4.62 | **P+ better** |
| P+ vs H1 | q4_p50_us | 16100 | 943.1 | 29099 | 1167 | 1061 | 3119 | +80.7% | 12.3 | **P+ better** |
| P+ vs H1 | q5_p50_us | 43823 | 9153 | 48299 | 47706 | 34348 | 9384 | +10.2% | 0.13 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q6_p50_us | 63352 | 2633 | 12232 | 1712 | 2221 | 5501 | -80.7% | 23.0 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 957.8 | 175.1 | 970.4 | 150.8 | 163.4 | 232.5 | +1.3% | 0.08 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 6880 | 1213 | 5316 | 1390 | 1305 | 8491 | -22.7% | 1.20 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c2_q2_p50_us | 759.5 | 88.2 | 952.7 | 92.9 | 90.6 | 320.6 | +25.4% | 2.13 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q1_p50_us | 1949 | 147.5 | 981.8 | 81.2 | 119.1 | 315.8 | -49.6% | 8.12 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 9767 | 1295 | 5473 | 1129 | 1215 | 1693 | -44.0% | 3.53 | **H1 better** |
| P+ vs H1 | c4_q2_p50_us | 1822 | 97.7 | 958.5 | 67.6 | 84.0 | 371.3 | -47.4% | 10.3 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 11107 | 2496 | 5027 | 1407 | 2026 | 325.3 | -54.7% | 3.00 | **H1 better** |
| P+ vs H1 | write_p50_us | 994.1 | 92.1 | 1652 | 112.7 | 102.9 | 205.8 | +66.2% | 6.39 | **P+ better** |
| P+ vs H1 | write_p99_us | 3331 | 200.1 | 5415 | 675.0 | 497.8 | 882.9 | +62.5% | 4.19 | **P+ better** |
| P+ vs H1 | commits_per_s | 814.9 | 73.3 | 502.9 | 45.3 | 60.9 | 61.8 | -38.3% | 5.12 | **P+ better** |
| P+ vs H1 | pss_mib | 44.4 | 1.17 | 422.3 | 9.08 | 6.48 | 1.86 | +850.7% | 58.3 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -469.5 | 950.2 | 15990 | 1929 | 1521 | 24996 | -3505.8% | 10.8 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 181.1 | 18.2 | 149.1 | 19.1 | 18.7 | 23.4 | -17.7% | 1.72 | no difference |
| N vs H2 | q1_p99_us | 674.6 | 97.8 | 603.4 | 174.3 | 141.3 | 147.3 | -10.6% | 0.50 | BELOW FLOOR |
| N vs H2 | q2_p50_us | 71426 | 4400 | 251.7 | 35.9 | 3111 | 6268 | -99.6% | 22.9 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 28108 | 4501 | 201.0 | 8.11 | 3182 | 2702 | -99.3% | 8.77 | **H2 better** |
| N vs H2 | q6_p50_us | 55681 | 1456 | 6760 | 1625 | 1543 | 456.6 | -87.9% | 31.7 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 165.1 | 28.0 | 135.0 | 16.6 | 23.0 | 63.8 | -18.2% | 1.31 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q1_p99_us | 3264 | 389.5 | 2201 | 639.2 | 529.2 | 2655 | -32.6% | 2.01 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 94520 | 9570 | 247.0 | 48.9 | 6767 | 13736 | -99.7% | 13.9 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 203.9 | 29.5 | 184.8 | 36.5 | 33.2 | 86.9 | -9.3% | 0.57 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p99_us | 7618 | 629.7 | 2865 | 812.3 | 726.8 | 1693 | -62.4% | 6.54 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 116399 | 7370 | 335.7 | 52.8 | 5212 | 12635 | -99.7% | 22.3 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 387609 | 15837 | 4447 | 937.5 | 11218 | 21870 | -98.9% | 34.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 553.9 | 12.2 | 1803 | 165.2 | 117.2 | 121.7 | +225.6% | 10.7 | **N better** |
| N vs H2 | write_p99_us | 1998 | 928.5 | 5678 | 909.3 | 918.9 | 564.2 | +184.2% | 4.00 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1502 | 156.5 | 489.6 | 37.3 | 113.7 | 353.5 | -67.4% | 8.90 | **N better** |
| N vs H2 | pss_mib | 187.3 | 25.0 | 43.7 | 0.03 | 17.7 | 25.5 | -76.7% | 8.12 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 57129 | 4792 | -738.6 | 325.0 | 3396 | 10752 | -101.3% | 17.0 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p50_us | 688.3 | 51.8 | 149.1 | 19.1 | 39.1 | 109.2 | -78.3% | 13.8 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2460 | 987.7 | 603.4 | 174.3 | 709.2 | 187.7 | -75.5% | 2.62 | no difference |
| P+ vs H2 | q2_p50_us | 545.1 | 14.9 | 251.7 | 35.9 | 27.4 | 37.6 | -53.8% | 10.7 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 327.4 | 46.4 | 289.2 | 29.2 | 38.8 | 63.7 | -11.7% | 0.98 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 16100 | 943.1 | 1222 | 118.3 | 672.1 | 645.9 | -92.4% | 22.1 | **H2 better** |
| P+ vs H2 | q5_p50_us | 43823 | 9153 | 201.0 | 8.11 | 6472 | 9344 | -99.5% | 6.74 | **H2 better** |
| P+ vs H2 | q6_p50_us | 63352 | 2633 | 6760 | 1625 | 2188 | 5515 | -89.3% | 25.9 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 957.8 | 175.1 | 135.0 | 16.6 | 124.4 | 226.1 | -85.9% | 6.61 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q1_p99_us | 6880 | 1213 | 2201 | 639.2 | 969.6 | 8155 | -68.0% | 4.83 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 759.5 | 88.2 | 247.0 | 48.9 | 71.3 | 326.5 | -67.5% | 7.19 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q1_p50_us | 1949 | 147.5 | 184.8 | 36.5 | 107.4 | 306.5 | -90.5% | 16.4 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 9767 | 1295 | 2865 | 812.3 | 1081 | 1143 | -70.7% | 6.38 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1822 | 97.7 | 335.7 | 52.8 | 78.5 | 316.3 | -81.6% | 18.9 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 11107 | 2496 | 4447 | 937.5 | 1885 | 2949 | -60.0% | 3.53 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | write_p50_us | 994.1 | 92.1 | 1803 | 165.2 | 133.8 | 142.8 | +81.4% | 6.05 | **P+ better** |
| P+ vs H2 | write_p99_us | 3331 | 200.1 | 5678 | 909.3 | 658.4 | 449.3 | +70.4% | 3.56 | **P+ better** |
| P+ vs H2 | commits_per_s | 814.9 | 73.3 | 489.6 | 37.3 | 58.2 | 62.5 | -39.9% | 5.59 | **P+ better** |
| P+ vs H2 | pss_mib | 44.4 | 1.17 | 43.7 | 0.03 | 0.83 | 0.22 | -1.7% | 0.90 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -469.5 | 950.2 | -738.6 | 325.0 | 710.1 | 2760 | +57.3% | 0.38 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 232.0 | 21.2 | 585.2 | 33.4 | 28.0 | 5.42 | +152.3% | 12.6 | **N better** |
| N vs P+ | q1_p99_us | 1399 | 555.1 | 4622 | 770.3 | 671.4 | 2288 | +230.3% | 4.80 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 687919 | 89609 | 508.3 | 26.5 | 63363 | 88251 | -99.9% | 10.8 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 126602 | 19699 | 143059 | 7046 | 14793 | 129236 | +13.0% | 1.11 | BELOW FLOOR |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 139.0 | 12.0 | 621.7 | 67.1 | 48.2 | 29.8 | +347.2% | 10.0 | **N better** |
| N vs P+ | c2_q1_p99_us | 2690 | 1518 | 4838 | 488.4 | 1128 | 10483 | +79.9% | 1.90 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 691680 | 87866 | 571.3 | 60.7 | 62131 | 43021 | -99.9% | 11.1 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 144.7 | 10.8 | 1682 | 146.6 | 103.9 | 167.9 | +1062.7% | 14.8 | **N better** |
| N vs P+ | c4_q1_p99_us | 6153 | 657.8 | 7937 | 1355 | 1065 | 1338 | +29.0% | 1.67 | no difference |
| N vs P+ | c4_q2_p50_us | 1315470 | 90312 | 1561 | 191.3 | 63860 | 53212 | -99.9% | 20.6 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 3108570 | 53923 | 8915 | 2622 | 38175 | 70442 | -99.7% | 81.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 922.8 | 59.6 | 1219 | 96.0 | 79.9 | 290.2 | +32.1% | 3.71 | **N better** |
| N vs P+ | write_p99_us | 2436 | 613.2 | 3658 | 321.6 | 489.6 | 6138 | +50.2% | 2.50 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 895.5 | 53.2 | 663.5 | 41.6 | 47.8 | 272.6 | -25.9% | 4.86 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 1242 | 12.5 | 494.5 | 0.51 | 8.87 | 0.57 | -60.2% | 84.3 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 286735 | 39083 | 52535 | 12859 | 29093 | 729415 | -81.7% | 8.05 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 232.0 | 21.2 | 340.3 | 19.6 | 20.4 | 10.1 | +46.7% | 5.31 | **N better** |
| N vs H3 | q1_p99_us | 1399 | 555.1 | 4142 | 2269 | 1652 | 2878 | +196.0% | 1.66 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 687919 | 89609 | 328.8 | 26.5 | 63363 | 88251 | -100.0% | 10.9 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 126602 | 19699 | 146224 | 8498 | 15170 | 129995 | +15.5% | 1.29 | BELOW FLOOR |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 139.0 | 12.0 | 286.7 | 43.1 | 31.7 | 22.1 | +106.2% | 4.66 | **N better** |
| N vs H3 | c2_q1_p99_us | 2690 | 1518 | 1970 | 893.7 | 1246 | 24073 | -26.8% | 0.58 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 691680 | 87866 | 288.2 | 40.0 | 62131 | 43021 | -100.0% | 11.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 144.7 | 10.8 | 383.4 | 43.4 | 31.6 | 58.1 | +165.0% | 7.55 | **N better** |
| N vs H3 | c4_q1_p99_us | 6153 | 657.8 | 3857 | 871.2 | 771.9 | 6344 | -37.3% | 2.97 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 1315470 | 90312 | 380.2 | 36.7 | 63860 | 53211 | -100.0% | 20.6 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 3108570 | 53923 | 5165 | 1289 | 38140 | 70434 | -99.8% | 81.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 922.8 | 59.6 | 1778 | 41.9 | 51.5 | 386.5 | +92.6% | 16.6 | **N better** |
| N vs H3 | write_p99_us | 2436 | 613.2 | 7693 | 3593 | 2577 | 6056 | +215.8% | 2.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 895.5 | 53.2 | 417.4 | 76.4 | 65.9 | 309.7 | -53.4% | 7.26 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | pss_mib | 1242 | 12.5 | 535.2 | 0.27 | 8.87 | 3.11 | -56.9% | 79.8 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 286735 | 39083 | 65373 | 16143 | 29900 | 738381 | -77.2% | 7.40 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 585.2 | 33.4 | 340.3 | 19.6 | 27.4 | 11.3 | -41.8% | 8.94 | **H3 better** |
| P+ vs H3 | q1_p99_us | 4622 | 770.3 | 4142 | 2269 | 1694 | 3665 | -10.4% | 0.28 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 508.3 | 26.5 | 328.8 | 26.5 | 26.5 | 30.2 | -35.3% | 6.76 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 295.8 | 36.1 | 376.7 | 60.3 | 49.7 | 218.6 | +27.4% | 1.63 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | 87153 | 1235 | 91918 | 2219 | 1796 | 4326 | +5.5% | 2.65 | no difference |
| P+ vs H3 | q5_p50_us | 143059 | 7046 | 146224 | 8498 | 7806 | 73808 | +2.2% | 0.41 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 621.7 | 67.1 | 286.7 | 43.1 | 56.4 | 35.1 | -53.9% | 5.94 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4838 | 488.4 | 1970 | 893.7 | 720.2 | 25754 | -59.3% | 3.98 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 571.3 | 60.7 | 288.2 | 40.0 | 51.4 | 97.5 | -49.6% | 5.51 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1682 | 146.6 | 383.4 | 43.4 | 108.1 | 167.6 | -77.2% | 12.0 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 7937 | 1355 | 3857 | 871.2 | 1139 | 6314 | -51.4% | 3.58 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1561 | 191.3 | 380.2 | 36.7 | 137.8 | 295.0 | -75.6% | 8.57 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8915 | 2622 | 5165 | 1289 | 2066 | 2131 | -42.1% | 1.81 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1219 | 96.0 | 1778 | 41.9 | 74.1 | 338.7 | +45.8% | 7.55 | **P+ better** |
| P+ vs H3 | write_p99_us | 3658 | 321.6 | 7693 | 3593 | 2551 | 2763 | +110.3% | 1.58 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 663.5 | 41.6 | 417.4 | 76.4 | 61.5 | 198.8 | -37.1% | 4.00 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | pss_mib | 494.5 | 0.51 | 535.2 | 0.27 | 0.41 | 3.07 | +8.2% | 100.0 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 52535 | 12859 | 65373 | 16143 | 14594 | 225971 | +24.4% | 0.88 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 232.0 | 21.2 | 157.2 | 11.4 | 17.0 | 49.3 | -32.3% | 4.40 | **H2 better** |
| N vs H2 | q1_p99_us | 1399 | 555.1 | 1603 | 739.7 | 653.9 | 398.5 | +14.6% | 0.31 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 687919 | 89609 | 292.7 | 20.0 | 63363 | 88251 | -100.0% | 10.9 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 126602 | 19699 | 208.3 | 13.2 | 13929 | 118644 | -99.8% | 9.07 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 139.0 | 12.0 | 68.2 | 7.38 | 9.94 | 17.5 | -50.9% | 7.12 | **H2 better** |
| N vs H2 | c2_q1_p99_us | 2690 | 1518 | 1176 | 80.1 | 1075 | 3636 | -56.3% | 1.41 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 691680 | 87866 | 202.4 | 23.9 | 62131 | 43021 | -100.0% | 11.1 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 144.7 | 10.8 | 156.4 | 30.0 | 22.6 | 137.9 | +8.1% | 0.52 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p99_us | 6153 | 657.8 | 3172 | 747.3 | 704.0 | 1055 | -48.5% | 4.24 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 1315470 | 90312 | 328.8 | 47.9 | 63860 | 53211 | -100.0% | 20.6 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 3108570 | 53923 | 16218 | 7905 | 38537 | 70446 | -99.5% | 80.2 | **H2 better** |
| N vs H2 | write_p50_us | 922.8 | 59.6 | 2103 | 85.7 | 73.8 | 345.4 | +127.9% | 16.0 | **N better** |
| N vs H2 | write_p99_us | 2436 | 613.2 | 5893 | 572.5 | 593.2 | 9039 | +141.9% | 5.83 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | commits_per_s | 895.5 | 53.2 | 420.0 | 18.2 | 39.8 | 293.0 | -53.1% | 12.0 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | pss_mib | 1242 | 12.5 | 492.1 | 0.47 | 8.87 | 3.78 | -60.4% | 84.6 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 286735 | 39083 | 53845 | 13146 | 29157 | 730560 | -81.2% | 7.99 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 585.2 | 33.4 | 157.2 | 11.4 | 24.9 | 49.5 | -73.1% | 17.2 | **H2 better** |
| P+ vs H2 | q1_p99_us | 4622 | 770.3 | 1603 | 739.7 | 755.1 | 2305 | -65.3% | 4.00 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q2_p50_us | 508.3 | 26.5 | 292.7 | 20.0 | 23.5 | 98.9 | -42.4% | 9.17 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 295.8 | 36.1 | 360.3 | 41.2 | 38.7 | 222.8 | +21.8% | 1.67 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q4_p50_us | 87153 | 1235 | 9025 | 382.5 | 914.2 | 4186 | -89.6% | 85.5 | **H2 better** |
| P+ vs H2 | q5_p50_us | 143059 | 7046 | 208.3 | 13.2 | 4982 | 51239 | -99.9% | 28.7 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 621.7 | 67.1 | 68.2 | 7.38 | 47.7 | 32.3 | -89.0% | 11.6 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 4838 | 488.4 | 1176 | 80.1 | 350.0 | 9848 | -75.7% | 10.5 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 571.3 | 60.7 | 202.4 | 23.9 | 46.2 | 70.7 | -64.6% | 7.99 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1682 | 146.6 | 156.4 | 30.0 | 105.8 | 209.1 | -90.7% | 14.4 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 7937 | 1355 | 3172 | 747.3 | 1094 | 855.2 | -60.0% | 4.35 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1561 | 191.3 | 328.8 | 47.9 | 139.5 | 289.6 | -78.9% | 8.83 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 8915 | 2622 | 16218 | 7905 | 5890 | 2505 | +81.9% | 1.24 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 1219 | 96.0 | 2103 | 85.7 | 91.0 | 291.0 | +72.5% | 9.72 | **P+ better** |
| P+ vs H2 | write_p99_us | 3658 | 321.6 | 5893 | 572.5 | 464.3 | 7257 | +61.1% | 4.81 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | commits_per_s | 663.5 | 41.6 | 420.0 | 18.2 | 32.1 | 171.5 | -36.7% | 7.59 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | pss_mib | 494.5 | 0.51 | 492.1 | 0.47 | 0.49 | 3.75 | -0.5% | 5.02 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | 52535 | 12859 | 53845 | 13146 | 13004 | 198932 | +2.5% | 0.10 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 274.1 | 9.97 | 681.9 | 29.2 | 21.8 | 87.0 | +148.8% | 18.7 | **N better** |
| N vs P+ | q1_p99_us | 2295 | 1063 | 4203 | 911.3 | 990.1 | 713.4 | +83.2% | 1.93 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 779619 | 66221 | 562.6 | 16.9 | 46826 | 52217 | -99.9% | 16.6 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 515974 | 364508 | 704348 | 543503 | 462743 | 48754 | +36.5% | 0.41 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q6_p50_us | 1799727 | 32349 | 2373409 | 43292 | 38214 | 185922 | +31.9% | 15.0 | **N better** |
| N vs P+ | c2_q1_p50_us | 154.4 | 30.2 | 637.0 | 91.7 | 68.3 | 21.6 | +312.6% | 7.07 | **N better** |
| N vs P+ | c2_q1_p99_us | 1511 | 709.8 | 5587 | 819.9 | 766.8 | 3748 | +269.7% | 5.32 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 806808 | 33215 | 563.9 | 70.7 | 23487 | 91728 | -99.9% | 34.3 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 133.6 | 12.6 | 1742 | 136.6 | 97.0 | 88.6 | +1204.0% | 16.6 | **N better** |
| N vs P+ | c4_q1_p99_us | 6355 | 621.9 | 9442 | 1590 | 1207 | 2202 | +48.6% | 2.56 | no difference |
| N vs P+ | c4_q2_p50_us | 1284122 | 73893 | 1573 | 101.0 | 52250 | 458251 | -99.9% | 24.5 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 3279147 | 289188 | 10929 | 2924 | 204497 | 79529 | -99.7% | 16.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 966.9 | 82.3 | 1334 | 52.8 | 69.1 | 3393 | +38.0% | 5.31 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 2468 | 306.8 | 4589 | 743.6 | 568.8 | 2831 | +85.9% | 3.73 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 884.4 | 87.0 | 632.5 | 20.7 | 63.2 | 333.6 | -28.5% | 3.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 2192 | 12.3 | 496.0 | 0.32 | 8.70 | 55.6 | -77.4% | 195.0 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 494578 | 119547 | 55701 | 14153 | 85123 | 977987 | -88.7% | 5.16 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 274.1 | 9.97 | 384.0 | 20.0 | 15.8 | 10.4 | +40.1% | 6.95 | **N better** |
| N vs H3 | q1_p99_us | 2295 | 1063 | 10877 | 3031 | 2271 | 3802 | +374.0% | 3.78 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 779619 | 66221 | 336.9 | 28.2 | 46826 | 52217 | -100.0% | 16.6 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 515974 | 364508 | 682236 | 547042 | 464823 | 54009 | +32.2% | 0.36 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q6_p50_us | 1799727 | 32349 | 2407896 | 57837 | 46859 | 259221 | +33.8% | 13.0 | **N better** |
| N vs H3 | c2_q1_p50_us | 154.4 | 30.2 | 315.3 | 36.4 | 33.5 | 19.6 | +104.2% | 4.81 | **N better** |
| N vs H3 | c2_q1_p99_us | 1511 | 709.8 | 2434 | 365.6 | 564.6 | 11522 | +61.0% | 1.63 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 806808 | 33215 | 302.4 | 34.1 | 23487 | 91728 | -100.0% | 34.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 133.6 | 12.6 | 386.7 | 77.5 | 55.5 | 67.8 | +189.4% | 4.56 | **N better** |
| N vs H3 | c4_q1_p99_us | 6355 | 621.9 | 3362 | 1022 | 846.2 | 5311 | -47.1% | 3.54 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 1284122 | 73893 | 446.5 | 71.5 | 52250 | 458251 | -100.0% | 24.6 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 3279147 | 289188 | 4601 | 2584 | 204495 | 79445 | -99.9% | 16.0 | **H3 better** |
| N vs H3 | write_p50_us | 966.9 | 82.3 | 1898 | 53.6 | 69.5 | 3392 | +96.2% | 13.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p99_us | 2468 | 306.8 | 5334 | 901.0 | 673.0 | 2240 | +116.1% | 4.26 | **N better** |
| N vs H3 | commits_per_s | 884.4 | 87.0 | 409.5 | 68.7 | 78.4 | 329.1 | -53.7% | 6.06 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 2192 | 12.3 | 562.2 | 0.29 | 8.70 | 55.8 | -74.4% | 187.4 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 494578 | 119547 | 70648 | 17707 | 85455 | 988858 | -85.7% | 4.96 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 681.9 | 29.2 | 384.0 | 20.0 | 25.0 | 87.5 | -43.7% | 11.9 | **H3 better** |
| P+ vs H3 | q1_p99_us | 4203 | 911.3 | 10877 | 3031 | 2238 | 3758 | +158.8% | 2.98 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 562.6 | 16.9 | 336.9 | 28.2 | 23.3 | 71.9 | -40.1% | 9.71 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 332.7 | 27.1 | 396.5 | 65.8 | 50.4 | 70.8 | +19.2% | 1.27 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 98528 | 3351 | 94557 | 2777 | 3077 | 4457 | -4.0% | 1.29 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 704348 | 543503 | 682236 | 547042 | 545276 | 23564 | -3.1% | 0.04 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 2373409 | 43292 | 2407896 | 57837 | 51085 | 246279 | +1.5% | 0.68 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 637.0 | 91.7 | 315.3 | 36.4 | 69.8 | 18.8 | -50.5% | 4.61 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5587 | 819.9 | 2434 | 365.6 | 634.8 | 12108 | -56.4% | 4.97 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 563.9 | 70.7 | 302.4 | 34.1 | 55.5 | 137.0 | -46.4% | 4.71 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1742 | 136.6 | 386.7 | 77.5 | 111.0 | 108.0 | -77.8% | 12.2 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 9442 | 1590 | 3362 | 1022 | 1336 | 5362 | -64.4% | 4.55 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1573 | 101.0 | 446.5 | 71.5 | 87.5 | 132.0 | -71.6% | 12.9 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 10929 | 2924 | 4601 | 2584 | 2759 | 3666 | -57.9% | 2.29 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1334 | 52.8 | 1898 | 53.6 | 53.2 | 86.5 | +42.2% | 10.6 | **P+ better** |
| P+ vs H3 | write_p99_us | 4589 | 743.6 | 5334 | 901.0 | 826.0 | 2431 | +16.2% | 0.90 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 632.5 | 20.7 | 409.5 | 68.7 | 50.7 | 75.8 | -35.2% | 4.39 | **P+ better** |
| P+ vs H3 | pss_mib | 496.0 | 0.32 | 562.2 | 0.29 | 0.30 | 4.74 | +13.3% | 218.1 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 55701 | 14153 | 70648 | 17707 | 16029 | 224514 | +26.8% | 0.93 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 274.1 | 9.97 | 151.0 | 2.62 | 7.29 | 20.7 | -44.9% | 16.9 | **H2 better** |
| N vs H2 | q1_p99_us | 2295 | 1063 | 1461 | 402.4 | 803.7 | 745.2 | -36.3% | 1.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 779619 | 66221 | 334.0 | 29.2 | 46826 | 52217 | -100.0% | 16.6 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 515974 | 364508 | 220.9 | 31.8 | 257746 | 48676 | -100.0% | 2.00 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | q6_p50_us | 1799727 | 32349 | 41577 | 1158 | 22889 | 143763 | -97.7% | 76.8 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 154.4 | 30.2 | 68.5 | 5.67 | 21.8 | 29.4 | -55.6% | 3.95 | **H2 better** |
| N vs H2 | c2_q1_p99_us | 1511 | 709.8 | 1282 | 122.2 | 509.3 | 1278 | -15.2% | 0.45 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q2_p50_us | 806808 | 33215 | 196.4 | 30.7 | 23487 | 91728 | -100.0% | 34.3 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 133.6 | 12.6 | 151.3 | 20.3 | 16.9 | 52.4 | +13.3% | 1.05 | BELOW FLOOR |
| N vs H2 | c4_q1_p99_us | 6355 | 621.9 | 3339 | 173.1 | 456.5 | 1556 | -47.5% | 6.61 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 1284122 | 73893 | 320.3 | 63.7 | 52250 | 458251 | -100.0% | 24.6 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p99_us | 3279147 | 289188 | 8904 | 2397 | 204494 | 79454 | -99.7% | 16.0 | **H2 better** |
| N vs H2 | write_p50_us | 966.9 | 82.3 | 2181 | 171.9 | 134.8 | 3395 | +125.6% | 9.01 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | write_p99_us | 2468 | 306.8 | 5938 | 456.6 | 389.0 | 2051 | +140.6% | 8.92 | **N better** |
| N vs H2 | commits_per_s | 884.4 | 87.0 | 398.8 | 33.9 | 66.0 | 336.3 | -54.9% | 7.35 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_mib | 2192 | 12.3 | 492.3 | 0.94 | 8.72 | 55.7 | -77.5% | 194.9 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 494578 | 119547 | 55002 | 13897 | 85102 | 980038 | -88.9% | 5.17 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 681.9 | 29.2 | 151.0 | 2.62 | 20.7 | 89.3 | -77.9% | 25.6 | **H2 better** |
| P+ vs H2 | q1_p99_us | 4203 | 911.3 | 1461 | 402.4 | 704.4 | 472.9 | -65.2% | 3.89 | **H2 better** |
| P+ vs H2 | q2_p50_us | 562.6 | 16.9 | 334.0 | 29.2 | 23.8 | 93.9 | -40.6% | 9.59 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 332.7 | 27.1 | 281.4 | 33.1 | 30.3 | 85.7 | -15.4% | 1.70 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 98528 | 3351 | 8963 | 196.2 | 2374 | 3072 | -90.9% | 37.7 | **H2 better** |
| P+ vs H2 | q5_p50_us | 704348 | 543503 | 220.9 | 31.8 | 384315 | 2761 | -100.0% | 1.83 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q6_p50_us | 2373409 | 43292 | 41577 | 1158 | 30623 | 118852 | -98.2% | 76.1 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 637.0 | 91.7 | 68.5 | 5.67 | 65.0 | 28.9 | -89.2% | 8.75 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5587 | 819.9 | 1282 | 122.2 | 586.2 | 3934 | -77.1% | 7.35 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 563.9 | 70.7 | 196.4 | 30.7 | 54.5 | 104.2 | -65.2% | 6.74 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1742 | 136.6 | 151.3 | 20.3 | 97.6 | 99.1 | -91.3% | 16.3 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 9442 | 1590 | 3339 | 173.1 | 1131 | 1724 | -64.6% | 5.40 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1573 | 101.0 | 320.3 | 63.7 | 84.4 | 180.3 | -79.6% | 14.8 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p99_us | 10929 | 2924 | 8904 | 2397 | 2673 | 3859 | -18.5% | 0.76 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 1334 | 52.8 | 2181 | 171.9 | 127.2 | 161.7 | +63.5% | 6.66 | **P+ better** |
| P+ vs H2 | write_p99_us | 4589 | 743.6 | 5938 | 456.6 | 617.0 | 2258 | +29.4% | 2.19 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 632.5 | 20.7 | 398.8 | 33.9 | 28.1 | 102.6 | -36.9% | 8.32 | **P+ better** |
| P+ vs H2 | pss_mib | 496.0 | 0.32 | 492.3 | 0.94 | 0.70 | 3.25 | -0.8% | 5.36 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | 55701 | 14153 | 55002 | 13897 | 14025 | 181772 | -1.3% | 0.05 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 232.2 | 26.0 | 609.4 | 23.8 | 24.9 | 137.5 | +162.4% | 15.1 | **N better** |
| N vs P+ | q1_p99_us | 1501 | 605.4 | 3789 | 1008 | 831.7 | 368.2 | +152.4% | 2.75 | no difference |
| N vs P+ | q2_p50_us | 700270 | 46645 | 523.0 | 35.6 | 32983 | 92816 | -99.9% | 21.2 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 157999 | 45117 | 179335 | 37333 | 41408 | 179224 | +13.5% | 0.52 | BELOW FLOOR |
| N vs P+ | q6_p50_us | 1635266 | 71192 | 2280471 | 64446 | 67903 | 82028 | +39.5% | 9.50 | **N better** |
| N vs P+ | c2_q1_p50_us | 138.4 | 21.6 | 600.9 | 58.0 | 43.8 | 8.66 | +334.3% | 10.6 | **N better** |
| N vs P+ | c2_q1_p99_us | 1150 | 625.8 | 5261 | 349.8 | 506.9 | 807.7 | +357.4% | 8.11 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 741842 | 83928 | 540.4 | 31.1 | 59346 | 2534 | -99.9% | 12.5 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 150.0 | 18.5 | 1770 | 166.1 | 118.2 | 339.9 | +1080.4% | 13.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 6180 | 508.3 | 8011 | 1906 | 1395 | 1347 | +29.6% | 1.31 | no difference |
| N vs P+ | c4_q2_p50_us | 1244320 | 132582 | 1652 | 131.5 | 93749 | 296159 | -99.9% | 13.3 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 3206934 | 331875 | 8768 | 1533 | 234674 | 79627 | -99.7% | 13.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 1001 | 166.9 | 1186 | 83.7 | 132.0 | 1318 | +18.5% | 1.40 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 3056 | 1158 | 3981 | 392.4 | 864.5 | 2440 | +30.3% | 1.07 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 832.6 | 79.7 | 667.4 | 16.5 | 57.5 | 165.2 | -19.8% | 2.87 | no difference |
| N vs P+ | pss_mib | 1736 | 492.9 | 496.5 | 0.32 | 348.5 | 5.97 | -71.4% | 3.56 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 379192 | 33216 | 54825 | 13913 | 25464 | 713856 | -85.5% | 12.7 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs H3 | q1_p50_us | 232.2 | 26.0 | 352.8 | 17.5 | 22.2 | 46.0 | +51.9% | 5.43 | **N better** |
| N vs H3 | q1_p99_us | 1501 | 605.4 | 2682 | 1357 | 1051 | 2509 | +78.7% | 1.12 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 700270 | 46645 | 322.1 | 27.8 | 32983 | 92816 | -100.0% | 21.2 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 157999 | 45117 | 195407 | 47413 | 46280 | 126900 | +23.7% | 0.81 | BELOW FLOOR |
| N vs H3 | q6_p50_us | 1635266 | 71192 | 2374988 | 111763 | 93699 | 394838 | +45.2% | 7.89 | **N better** |
| N vs H3 | c2_q1_p50_us | 138.4 | 21.6 | 301.6 | 21.6 | 21.6 | 37.1 | +118.0% | 7.56 | **N better** |
| N vs H3 | c2_q1_p99_us | 1150 | 625.8 | 2424 | 1201 | 957.4 | 839.0 | +110.8% | 1.33 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 741842 | 83928 | 282.5 | 31.7 | 59346 | 2534 | -100.0% | 12.5 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 150.0 | 18.5 | 373.2 | 66.7 | 49.0 | 161.4 | +148.9% | 4.56 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 6180 | 508.3 | 5382 | 2506 | 1808 | 1073 | -12.9% | 0.44 | BELOW FLOOR |
| N vs H3 | c4_q2_p50_us | 1244320 | 132582 | 364.8 | 56.7 | 93749 | 296158 | -100.0% | 13.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 3206934 | 331875 | 5130 | 2557 | 234678 | 79532 | -99.8% | 13.6 | **H3 better** |
| N vs H3 | write_p50_us | 1001 | 166.9 | 1752 | 69.2 | 127.7 | 1327 | +75.1% | 5.88 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p99_us | 3056 | 1158 | 7022 | 1012 | 1087 | 821.9 | +129.8% | 3.65 | **N better** |
| N vs H3 | commits_per_s | 832.6 | 79.7 | 443.8 | 43.9 | 64.3 | 150.6 | -46.7% | 6.05 | **N better** |
| N vs H3 | pss_mib | 1736 | 492.9 | 553.8 | 18.9 | 348.8 | 0.90 | -68.1% | 3.39 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 379192 | 33216 | 69375 | 10392 | 24610 | 724972 | -81.7% | 12.6 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 609.4 | 23.8 | 352.8 | 17.5 | 20.9 | 131.2 | -42.1% | 12.3 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3789 | 1008 | 2682 | 1357 | 1195 | 2528 | -29.2% | 0.93 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 523.0 | 35.6 | 322.1 | 27.8 | 31.9 | 151.5 | -38.4% | 6.29 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 290.5 | 41.7 | 369.7 | 49.1 | 45.6 | 131.6 | +27.3% | 1.74 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q5_p50_us | 179335 | 37333 | 195407 | 47413 | 42672 | 193629 | +9.0% | 0.38 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 2280471 | 64446 | 2374988 | 111763 | 91225 | 386480 | +4.1% | 1.04 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 600.9 | 58.0 | 301.6 | 21.6 | 43.8 | 36.6 | -49.8% | 6.84 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5261 | 349.8 | 2424 | 1201 | 884.3 | 326.5 | -53.9% | 3.21 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 540.4 | 31.1 | 282.5 | 31.7 | 31.4 | 9.42 | -47.7% | 8.21 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1770 | 166.1 | 373.2 | 66.7 | 126.6 | 372.6 | -78.9% | 11.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 8011 | 1906 | 5382 | 2506 | 2226 | 1660 | -32.8% | 1.18 | no difference |
| P+ vs H3 | c4_q2_p50_us | 1652 | 131.5 | 364.8 | 56.7 | 101.2 | 489.2 | -77.9% | 12.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 8768 | 1533 | 5130 | 2557 | 2108 | 3881 | -41.5% | 1.73 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1186 | 83.7 | 1752 | 69.2 | 76.8 | 167.4 | +47.8% | 7.38 | **P+ better** |
| P+ vs H3 | write_p99_us | 3981 | 392.4 | 7022 | 1012 | 767.4 | 2573 | +76.4% | 3.96 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 667.4 | 16.5 | 443.8 | 43.9 | 33.2 | 163.5 | -33.5% | 6.74 | **P+ better** |
| P+ vs H3 | pss_mib | 496.5 | 0.32 | 553.8 | 18.9 | 13.3 | 5.92 | +11.5% | 4.29 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 54825 | 13913 | 69375 | 10392 | 12280 | 219031 | +26.5% | 1.18 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 232.2 | 26.0 | 148.4 | 13.2 | 20.6 | 52.8 | -36.1% | 4.06 | **H2 better** |
| N vs H2 | q1_p99_us | 1501 | 605.4 | 1695 | 607.6 | 606.5 | 143.6 | +12.9% | 0.32 | no difference |
| N vs H2 | q2_p50_us | 700270 | 46645 | 296.4 | 23.7 | 32983 | 92816 | -100.0% | 21.2 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 157999 | 45117 | 200.4 | 29.7 | 31903 | 73255 | -99.9% | 4.95 | **H2 better** |
| N vs H2 | q6_p50_us | 1635266 | 71192 | 46738 | 10898 | 50927 | 83096 | -97.1% | 31.2 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 138.4 | 21.6 | 87.4 | 21.2 | 21.4 | 12.5 | -36.8% | 2.38 | no difference |
| N vs H2 | c2_q1_p99_us | 1150 | 625.8 | 1357 | 343.3 | 504.7 | 2512 | +18.0% | 0.41 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 741842 | 83928 | 225.4 | 51.0 | 59346 | 2535 | -100.0% | 12.5 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 150.0 | 18.5 | 191.9 | 38.2 | 30.0 | 106.3 | +28.0% | 1.40 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p99_us | 6180 | 508.3 | 3695 | 791.6 | 665.2 | 433.5 | -40.2% | 3.73 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 1244320 | 132582 | 394.5 | 34.9 | 93749 | 296158 | -100.0% | 13.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p99_us | 3206934 | 331875 | 11294 | 6530 | 234716 | 81309 | -99.6% | 13.6 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 1001 | 166.9 | 2160 | 126.6 | 148.1 | 1317 | +115.8% | 7.82 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | write_p99_us | 3056 | 1158 | 5999 | 680.3 | 949.6 | 2069 | +96.3% | 3.10 | **N better** |
| N vs H2 | commits_per_s | 832.6 | 79.7 | 397.1 | 25.2 | 59.1 | 109.6 | -52.3% | 7.37 | **N better** |
| N vs H2 | pss_mib | 1736 | 492.9 | 492.7 | 1.26 | 348.6 | 1.54 | -71.6% | 3.57 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 379192 | 33216 | 54332 | 14186 | 25539 | 713992 | -85.7% | 12.7 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 609.4 | 23.8 | 148.4 | 13.2 | 19.2 | 133.7 | -75.6% | 24.0 | **H2 better** |
| P+ vs H2 | q1_p99_us | 3789 | 1008 | 1695 | 607.6 | 832.5 | 339.8 | -55.3% | 2.51 | no difference |
| P+ vs H2 | q2_p50_us | 523.0 | 35.6 | 296.4 | 23.7 | 30.2 | 126.4 | -43.3% | 7.50 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 290.5 | 41.7 | 285.7 | 23.2 | 33.7 | 130.5 | -1.7% | 0.14 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q5_p50_us | 179335 | 37333 | 200.4 | 29.7 | 26398 | 163569 | -99.9% | 6.79 | **H2 better** |
| P+ vs H2 | q6_p50_us | 2280471 | 64446 | 46738 | 10898 | 46217 | 19356 | -98.0% | 48.3 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 600.9 | 58.0 | 87.4 | 21.2 | 43.7 | 10.9 | -85.5% | 11.8 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5261 | 349.8 | 1357 | 343.3 | 346.5 | 2391 | -74.2% | 11.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q2_p50_us | 540.4 | 31.1 | 225.4 | 51.0 | 42.2 | 65.8 | -58.3% | 7.46 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1770 | 166.1 | 191.9 | 38.2 | 120.5 | 352.2 | -89.2% | 13.1 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 8011 | 1906 | 3695 | 791.6 | 1459 | 1338 | -53.9% | 2.96 | no difference |
| P+ vs H2 | c4_q2_p50_us | 1652 | 131.5 | 394.5 | 34.9 | 96.2 | 437.0 | -76.1% | 13.1 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p99_us | 8768 | 1533 | 11294 | 6530 | 4743 | 17346 | +28.8% | 0.53 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1186 | 83.7 | 2160 | 126.6 | 107.3 | 41.3 | +82.1% | 9.07 | **P+ better** |
| P+ vs H2 | write_p99_us | 3981 | 392.4 | 5999 | 680.3 | 555.4 | 3198 | +50.7% | 3.63 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 667.4 | 16.5 | 397.1 | 25.2 | 21.3 | 126.7 | -40.5% | 12.7 | **P+ better** |
| P+ vs H2 | pss_mib | 496.5 | 0.32 | 492.7 | 1.26 | 0.92 | 6.05 | -0.8% | 4.16 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | 54825 | 13913 | 54332 | 14186 | 14050 | 179374 | -0.9% | 0.04 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 226.0 | 22.3 | 608.0 | 31.8 | 27.4 | 15.4 | +169.1% | 13.9 | **N better** |
| N vs P+ | q1_p99_us | 2124 | 676.4 | 4342 | 775.1 | 727.4 | 1277 | +104.4% | 3.05 | **N better** |
| N vs P+ | q2_p50_us | 723622 | 90806 | 564.7 | 17.1 | 64209 | 34558 | -99.9% | 11.3 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 774001 | 114670 | 1181324 | 75691 | 97155 | 124200 | +52.6% | 4.19 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 147.3 | 18.7 | 630.5 | 64.3 | 47.4 | 51.5 | +328.1% | 10.2 | **N better** |
| N vs P+ | c2_q1_p99_us | 2163 | 1518 | 6213 | 1091 | 1322 | 348.6 | +187.2% | 3.06 | **N better** |
| N vs P+ | c2_q2_p50_us | 785304 | 53747 | 558.9 | 53.8 | 38005 | 64694 | -99.9% | 20.6 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 145.6 | 13.0 | 1954 | 252.2 | 178.6 | 229.4 | +1241.6% | 10.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 6018 | 243.2 | 8858 | 1033 | 750.2 | 4294 | +47.2% | 3.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 1224336 | 116622 | 1916 | 162.5 | 82464 | 82681 | -99.8% | 14.8 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 3229691 | 263003 | 8313 | 1872 | 185976 | 451010 | -99.7% | 17.3 | **P+ better** |
| N vs P+ | write_p50_us | 924.6 | 39.7 | 1195 | 92.5 | 71.2 | 858.7 | +29.3% | 3.80 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 2892 | 735.2 | 4687 | 954.4 | 851.9 | 1839 | +62.1% | 2.11 | BELOW FLOOR |
| N vs P+ | commits_per_s | 869.9 | 88.5 | 646.7 | 77.0 | 83.0 | 197.6 | -25.7% | 2.69 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 2495 | 16.2 | 495.1 | 0.69 | 11.5 | 931.3 | -80.2% | 174.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_growth_bytes_per_key_read | 575895 | 141852 | 54820 | 13731 | 100773 | 442455 | -90.5% | 5.17 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H3 | q1_p50_us | 226.0 | 22.3 | 363.4 | 15.8 | 19.3 | 20.0 | +60.8% | 7.12 | **N better** |
| N vs H3 | q1_p99_us | 2124 | 676.4 | 3311 | 608.6 | 643.4 | 5718 | +55.9% | 1.85 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 723622 | 90806 | 358.6 | 11.9 | 64209 | 34558 | -100.0% | 11.3 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 774001 | 114670 | 1166861 | 73189 | 96192 | 54449 | +50.8% | 4.08 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 147.3 | 18.7 | 367.8 | 64.0 | 47.1 | 143.0 | +149.7% | 4.68 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q1_p99_us | 2163 | 1518 | 2732 | 817.2 | 1219 | 5192 | +26.3% | 0.47 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 785304 | 53747 | 348.6 | 49.2 | 38005 | 64695 | -100.0% | 20.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 145.6 | 13.0 | 459.8 | 43.2 | 31.9 | 210.1 | +215.8% | 9.85 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 6018 | 243.2 | 3998 | 954.9 | 696.8 | 4719 | -33.6% | 2.90 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 1224336 | 116622 | 464.2 | 64.0 | 82464 | 82681 | -100.0% | 14.8 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 3229691 | 263003 | 3921 | 1464 | 185974 | 451015 | -99.9% | 17.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 924.6 | 39.7 | 1721 | 73.8 | 59.2 | 855.1 | +86.2% | 13.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p99_us | 2892 | 735.2 | 5475 | 466.1 | 615.6 | 1972 | +89.3% | 4.20 | **N better** |
| N vs H3 | commits_per_s | 869.9 | 88.5 | 492.4 | 28.8 | 65.8 | 208.8 | -43.4% | 5.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 2495 | 16.2 | 584.0 | 0.27 | 11.5 | 931.3 | -76.6% | 166.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_growth_bytes_per_key_read | 575895 | 141852 | 75980 | 18658 | 101168 | 456467 | -86.8% | 4.94 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 608.0 | 31.8 | 363.4 | 15.8 | 25.1 | 12.8 | -40.2% | 9.75 | **H3 better** |
| P+ vs H3 | q1_p99_us | 4342 | 775.1 | 3311 | 608.6 | 696.8 | 5846 | -23.7% | 1.48 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 564.7 | 17.1 | 358.6 | 11.9 | 14.7 | 9.45 | -36.5% | 14.0 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 332.1 | 36.9 | 379.9 | 24.4 | 31.3 | 52.1 | +14.4% | 1.53 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 96250 | 4662 | 101685 | 3062 | 3944 | 13996 | +5.6% | 1.38 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 1181324 | 75691 | 1166861 | 73189 | 74451 | 114325 | -1.2% | 0.19 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 630.5 | 64.3 | 367.8 | 64.0 | 64.2 | 144.4 | -41.7% | 4.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 6213 | 1091 | 2732 | 817.2 | 963.7 | 5204 | -56.0% | 3.61 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 558.9 | 53.8 | 348.6 | 49.2 | 51.6 | 114.7 | -37.6% | 4.08 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1954 | 252.2 | 459.8 | 43.2 | 181.0 | 291.0 | -76.5% | 8.26 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 8858 | 1033 | 3998 | 954.9 | 994.6 | 2011 | -54.9% | 4.89 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1916 | 162.5 | 464.2 | 64.0 | 123.5 | 285.3 | -75.8% | 11.8 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8313 | 1872 | 3921 | 1464 | 1680 | 2364 | -52.8% | 2.61 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1195 | 92.5 | 1721 | 73.8 | 83.7 | 147.6 | +44.0% | 6.29 | **P+ better** |
| P+ vs H3 | write_p99_us | 4687 | 954.4 | 5475 | 466.1 | 751.0 | 986.9 | +16.8% | 1.05 | BELOW FLOOR |
| P+ vs H3 | commits_per_s | 646.7 | 77.0 | 492.4 | 28.8 | 58.1 | 71.3 | -23.9% | 2.65 | no difference |
| P+ vs H3 | pss_mib | 495.1 | 0.69 | 584.0 | 0.27 | 0.53 | 4.23 | +18.0% | 168.8 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 54820 | 13731 | 75980 | 18658 | 16381 | 233330 | +38.6% | 1.29 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 226.0 | 22.3 | 156.7 | 6.67 | 16.4 | 89.5 | -30.7% | 4.21 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q1_p99_us | 2124 | 676.4 | 1602 | 585.0 | 632.3 | 292.4 | -24.6% | 0.83 | no difference |
| N vs H2 | q2_p50_us | 723622 | 90806 | 323.7 | 26.9 | 64209 | 34558 | -100.0% | 11.3 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 774001 | 114670 | 209.0 | 34.9 | 81084 | 51576 | -100.0% | 9.54 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 147.3 | 18.7 | 140.7 | 19.8 | 19.3 | 55.3 | -4.4% | 0.34 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q1_p99_us | 2163 | 1518 | 1859 | 353.3 | 1102 | 3386 | -14.1% | 0.28 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q2_p50_us | 785304 | 53747 | 300.8 | 58.6 | 38005 | 64694 | -100.0% | 20.7 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 145.6 | 13.0 | 212.9 | 37.2 | 27.9 | 106.0 | +46.2% | 2.41 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q1_p99_us | 6018 | 243.2 | 3600 | 405.8 | 334.5 | 4320 | -40.2% | 7.23 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 1224336 | 116622 | 406.9 | 64.8 | 82464 | 82681 | -100.0% | 14.8 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p99_us | 3229691 | 263003 | 12192 | 2993 | 185984 | 451033 | -99.6% | 17.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 924.6 | 39.7 | 2196 | 36.5 | 38.1 | 851.1 | +137.6% | 33.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | write_p99_us | 2892 | 735.2 | 6917 | 1917 | 1452 | 1778 | +139.2% | 2.77 | no difference |
| N vs H2 | commits_per_s | 869.9 | 88.5 | 388.9 | 31.4 | 66.4 | 197.3 | -55.3% | 7.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_mib | 2495 | 16.2 | 492.9 | 0.82 | 11.5 | 931.3 | -80.2% | 174.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_growth_bytes_per_key_read | 575895 | 141852 | 54354 | 13310 | 100745 | 439225 | -90.6% | 5.18 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p50_us | 608.0 | 31.8 | 156.7 | 6.67 | 22.9 | 88.2 | -74.2% | 19.7 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p99_us | 4342 | 775.1 | 1602 | 585.0 | 686.6 | 1253 | -63.1% | 3.99 | **H2 better** |
| P+ vs H2 | q2_p50_us | 564.7 | 17.1 | 323.7 | 26.9 | 22.6 | 6.85 | -42.7% | 10.7 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 332.1 | 36.9 | 317.4 | 15.7 | 28.4 | 55.1 | -4.4% | 0.52 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 96250 | 4662 | 8858 | 280.0 | 3302 | 10470 | -90.8% | 26.5 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q5_p50_us | 1181324 | 75691 | 209.0 | 34.9 | 53522 | 112985 | -100.0% | 22.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 630.5 | 64.3 | 140.7 | 19.8 | 47.6 | 58.8 | -77.7% | 10.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q1_p99_us | 6213 | 1091 | 1859 | 353.3 | 810.7 | 3404 | -70.1% | 5.37 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q2_p50_us | 558.9 | 53.8 | 300.8 | 58.6 | 56.3 | 52.7 | -46.2% | 4.59 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1954 | 252.2 | 212.9 | 37.2 | 180.3 | 227.5 | -89.1% | 9.66 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 8858 | 1033 | 3600 | 405.8 | 784.6 | 661.7 | -59.4% | 6.70 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1916 | 162.5 | 406.9 | 64.8 | 123.7 | 313.3 | -78.8% | 12.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p99_us | 8313 | 1872 | 12192 | 2993 | 2496 | 4762 | +46.7% | 1.55 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | write_p50_us | 1195 | 92.5 | 2196 | 36.5 | 70.3 | 122.4 | +83.8% | 14.2 | **P+ better** |
| P+ vs H2 | write_p99_us | 4687 | 954.4 | 6917 | 1917 | 1514 | 495.7 | +47.6% | 1.47 | no difference |
| P+ vs H2 | commits_per_s | 646.7 | 77.0 | 388.9 | 31.4 | 58.8 | 20.1 | -39.9% | 4.38 | **P+ better** |
| P+ vs H2 | pss_mib | 495.1 | 0.69 | 492.9 | 0.82 | 0.76 | 1.12 | -0.4% | 2.84 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | 54820 | 13731 | 54354 | 13310 | 13522 | 197481 | -0.8% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `multi`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 264.8 | 23.6 | 647.4 | 34.2 | 29.4 | 45.9 | +144.5% | 13.0 | **N better** |
| N vs P+ | q1_p99_us | 2927 | 534.4 | 4819 | 433.2 | 486.4 | 320.5 | +64.6% | 3.89 | **N better** |
| N vs P+ | q2_p50_us | 766684 | 72178 | 545.9 | 40.9 | 51038 | 84643 | -99.9% | 15.0 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 428891 | 300707 | 638467 | 494430 | 409198 | 64649 | +48.9% | 0.51 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q6_p50_us | 1696662 | 57067 | 2294074 | 64442 | 60866 | 140783 | +35.2% | 9.82 | **N better** |
| N vs P+ | c2_q1_p50_us | 155.1 | 20.2 | 642.6 | 51.2 | 38.9 | 57.8 | +314.3% | 12.5 | **N better** |
| N vs P+ | c2_q1_p99_us | 1302 | 460.0 | 4939 | 342.4 | 405.5 | 2227 | +279.3% | 8.97 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 770722 | 63042 | 575.4 | 35.0 | 44578 | 116342 | -99.9% | 17.3 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 163.0 | 10.7 | 1671 | 93.2 | 66.3 | 91.4 | +925.5% | 22.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 5883 | 204.7 | 10837 | 1169 | 839.3 | 342.2 | +84.2% | 5.90 | **N better** |
| N vs P+ | c4_q2_p50_us | 1344117 | 39993 | 1551 | 103.5 | 28279 | 382580 | -99.9% | 47.5 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 3421148 | 220830 | 10164 | 1986 | 156157 | 410721 | -99.7% | 21.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 1051 | 119.9 | 1222 | 101.3 | 111.0 | 2223 | +16.3% | 1.54 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 2860 | 539.1 | 4384 | 632.1 | 587.4 | 7809 | +53.3% | 2.60 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 822.7 | 51.0 | 626.5 | 49.3 | 50.1 | 386.0 | -23.9% | 3.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 1913 | 246.9 | 493.2 | 1.33 | 174.6 | 72.0 | -74.2% | 8.13 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 428210 | 48794 | 55317 | 13130 | 35730 | 170590 | -87.1% | 10.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H3 | q1_p50_us | 264.8 | 23.6 | 371.4 | 22.9 | 23.2 | 40.4 | +40.3% | 4.59 | **N better** |
| N vs H3 | q1_p99_us | 2927 | 534.4 | 3062 | 1279 | 980.4 | 1252 | +4.6% | 0.14 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 766684 | 72178 | 340.3 | 35.5 | 51038 | 84643 | -100.0% | 15.0 | **H3 better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 428891 | 300707 | 684561 | 524518 | 427519 | 30075 | +59.6% | 0.60 | no difference |
| N vs H3 | q6_p50_us | 1696662 | 57067 | 2445426 | 67789 | 62657 | 244462 | +44.1% | 12.0 | **N better** |
| N vs H3 | c2_q1_p50_us | 155.1 | 20.2 | 296.4 | 26.8 | 23.7 | 42.4 | +91.1% | 5.95 | **N better** |
| N vs H3 | c2_q1_p99_us | 1302 | 460.0 | 2844 | 1553 | 1145 | 3850 | +118.4% | 1.35 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 770722 | 63042 | 280.9 | 34.2 | 44578 | 116342 | -100.0% | 17.3 | **H3 better** |
| N vs H3 | c4_q1_p50_us | 163.0 | 10.7 | 365.2 | 30.4 | 22.8 | 80.1 | +124.1% | 8.88 | **N better** |
| N vs H3 | c4_q1_p99_us | 5883 | 204.7 | 4537 | 2086 | 1482 | 1964 | -22.9% | 0.91 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 1344117 | 39993 | 400.7 | 50.6 | 28279 | 382580 | -100.0% | 47.5 | **H3 better** |
| N vs H3 | c4_q2_p99_us | 3421148 | 220830 | 4912 | 1348 | 156153 | 410738 | -99.9% | 21.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 1051 | 119.9 | 1961 | 64.4 | 96.2 | 2227 | +86.5% | 9.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p99_us | 2860 | 539.1 | 5380 | 818.0 | 692.7 | 8473 | +88.1% | 3.64 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 822.7 | 51.0 | 438.4 | 42.4 | 46.9 | 380.3 | -46.7% | 8.20 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 1913 | 246.9 | 533.1 | 0.43 | 174.6 | 72.0 | -72.1% | 7.90 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 428210 | 48794 | 64322 | 15715 | 36248 | 200398 | -85.0% | 10.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 647.4 | 34.2 | 371.4 | 22.9 | 29.1 | 29.4 | -42.6% | 9.48 | **H3 better** |
| P+ vs H3 | q1_p99_us | 4819 | 433.2 | 3062 | 1279 | 955.1 | 1211 | -36.5% | 1.84 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 545.9 | 40.9 | 340.3 | 35.5 | 38.3 | 166.3 | -37.7% | 5.37 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 347.3 | 43.5 | 434.0 | 38.4 | 41.0 | 185.7 | +25.0% | 2.11 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | 97752 | 5281 | 103013 | 5707 | 5498 | 17499 | +5.4% | 0.96 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 638467 | 494430 | 684561 | 524518 | 509696 | 58658 | +7.2% | 0.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q6_p50_us | 2294074 | 64442 | 2445426 | 67789 | 66136 | 273954 | +6.6% | 2.29 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 642.6 | 51.2 | 296.4 | 26.8 | 40.9 | 69.7 | -53.9% | 8.47 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4939 | 342.4 | 2844 | 1553 | 1124 | 4174 | -42.4% | 1.86 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 575.4 | 35.0 | 280.9 | 34.2 | 34.6 | 101.4 | -51.2% | 8.51 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1671 | 93.2 | 365.2 | 30.4 | 69.3 | 118.9 | -78.1% | 18.8 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 10837 | 1169 | 4537 | 2086 | 1691 | 1963 | -58.1% | 3.72 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1551 | 103.5 | 400.7 | 50.6 | 81.5 | 186.7 | -74.2% | 14.1 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 10164 | 1986 | 4912 | 1348 | 1697 | 5300 | -51.7% | 3.09 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1222 | 101.3 | 1961 | 64.4 | 84.9 | 155.0 | +60.4% | 8.70 | **P+ better** |
| P+ vs H3 | write_p99_us | 4384 | 632.1 | 5380 | 818.0 | 731.0 | 3296 | +22.7% | 1.36 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 626.5 | 49.3 | 438.4 | 42.4 | 46.0 | 140.0 | -30.0% | 4.09 | **P+ better** |
| P+ vs H3 | pss_mib | 493.2 | 1.33 | 533.1 | 0.43 | 0.99 | 1.49 | +8.1% | 40.4 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 55317 | 13130 | 64322 | 15715 | 14480 | 219682 | +16.3% | 0.62 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 264.8 | 23.6 | 163.6 | 20.2 | 22.0 | 103.3 | -38.2% | 4.61 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q1_p99_us | 2927 | 534.4 | 1661 | 750.3 | 651.4 | 320.4 | -43.3% | 1.94 | no difference |
| N vs H2 | q2_p50_us | 766684 | 72178 | 322.6 | 25.3 | 51038 | 84643 | -100.0% | 15.0 | **H2 better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 428891 | 300707 | 275.8 | 56.5 | 212632 | 28664 | -99.9% | 2.02 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | 1696662 | 57067 | 44902 | 6783 | 40636 | 47876 | -97.4% | 40.6 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 155.1 | 20.2 | 158.5 | 24.0 | 22.2 | 53.1 | +2.2% | 0.15 | BELOW FLOOR |
| N vs H2 | c2_q1_p99_us | 1302 | 460.0 | 2538 | 638.4 | 556.4 | 1486 | +94.9% | 2.22 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q2_p50_us | 770722 | 63042 | 285.2 | 49.8 | 44578 | 116342 | -100.0% | 17.3 | **H2 better** |
| N vs H2 | c4_q1_p50_us | 163.0 | 10.7 | 205.9 | 51.5 | 37.2 | 22.7 | +26.3% | 1.15 | no difference |
| N vs H2 | c4_q1_p99_us | 5883 | 204.7 | 3806 | 490.4 | 375.7 | 383.0 | -35.3% | 5.53 | **H2 better** |
| N vs H2 | c4_q2_p50_us | 1344117 | 39993 | 467.6 | 145.7 | 28279 | 382580 | -100.0% | 47.5 | **H2 better** |
| N vs H2 | c4_q2_p99_us | 3421148 | 220830 | 9070 | 4337 | 156180 | 410736 | -99.7% | 21.8 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 1051 | 119.9 | 2379 | 265.2 | 205.8 | 2229 | +126.3% | 6.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | write_p99_us | 2860 | 539.1 | 6213 | 605.3 | 573.2 | 7818 | +117.3% | 5.85 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 822.7 | 51.0 | 372.3 | 22.2 | 39.3 | 385.9 | -54.7% | 11.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_mib | 1913 | 246.9 | 490.4 | 1.50 | 174.6 | 72.0 | -74.4% | 8.15 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 428210 | 48794 | 54930 | 13353 | 35771 | 170600 | -87.2% | 10.4 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p50_us | 647.4 | 34.2 | 163.6 | 20.2 | 28.1 | 99.5 | -74.7% | 17.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p99_us | 4819 | 433.2 | 1661 | 750.3 | 612.6 | 10.8 | -65.5% | 5.16 | **H2 better** |
| P+ vs H2 | q2_p50_us | 545.9 | 40.9 | 322.6 | 25.3 | 34.0 | 162.1 | -40.9% | 6.56 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 347.3 | 43.5 | 350.2 | 35.4 | 39.6 | 190.0 | +0.8% | 0.07 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q4_p50_us | 97752 | 5281 | 10480 | 1876 | 3963 | 11820 | -89.3% | 22.0 | **H2 better** |
| P+ vs H2 | q5_p50_us | 638467 | 494430 | 275.8 | 56.5 | 349615 | 57947 | -100.0% | 1.83 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q6_p50_us | 2294074 | 64442 | 44902 | 6783 | 45819 | 132594 | -98.0% | 49.1 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 642.6 | 51.2 | 158.5 | 24.0 | 40.0 | 76.7 | -75.3% | 12.1 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 4939 | 342.4 | 2538 | 638.4 | 512.2 | 2192 | -48.6% | 4.69 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q2_p50_us | 575.4 | 35.0 | 285.2 | 49.8 | 43.1 | 139.1 | -50.4% | 6.74 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1671 | 93.2 | 205.9 | 51.5 | 75.3 | 90.7 | -87.7% | 19.5 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 10837 | 1169 | 3806 | 490.4 | 896.5 | 374.0 | -64.9% | 7.84 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1551 | 103.5 | 467.6 | 145.7 | 126.4 | 179.1 | -69.8% | 8.57 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 10164 | 1986 | 9070 | 4337 | 3373 | 5153 | -10.8% | 0.32 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1222 | 101.3 | 2379 | 265.2 | 200.8 | 174.5 | +94.6% | 5.76 | **P+ better** |
| P+ vs H2 | write_p99_us | 4384 | 632.1 | 6213 | 605.3 | 618.8 | 440.5 | +41.7% | 2.96 | no difference |
| P+ vs H2 | commits_per_s | 626.5 | 49.3 | 372.3 | 22.2 | 38.2 | 154.5 | -40.6% | 6.65 | **P+ better** |
| P+ vs H2 | pss_mib | 493.2 | 1.33 | 490.4 | 1.50 | 1.42 | 1.44 | -0.6% | 1.99 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | 55317 | 13130 | 54930 | 13353 | 13242 | 192887 | -0.7% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 107.6 | 20.6 | 701.5 | 52.4 | 39.8 | 46.6 | +552.1% | 14.9 | **N better** |
| N vs P+ | q1_p99_us | 511.1 | 72.0 | 2534 | 572.1 | 407.8 | 222.8 | +395.9% | 4.96 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 131.5 | 19.1 | 580.7 | 20.8 | 19.9 | 17.0 | +341.6% | 22.5 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2126 | 131.3 | 3472 | 373.0 | 279.7 | 81.0 | +63.3% | 4.81 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 117.7 | 21.2 | 997.4 | 165.1 | 117.7 | 366.6 | +747.1% | 7.47 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q1_p99_us | 654.7 | 293.3 | 7926 | 2456 | 1749 | 3609 | +1110.5% | 4.16 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 147.8 | 29.5 | 817.0 | 115.6 | 84.3 | 314.8 | +452.6% | 7.94 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 165.4 | 24.0 | 2042 | 122.1 | 88.0 | 125.7 | +1134.5% | 21.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 905.3 | 216.1 | 9239 | 2655 | 1884 | 1140 | +920.6% | 4.42 | **N better** |
| N vs P+ | c4_q2_p50_us | 223.8 | 21.9 | 1805 | 96.7 | 70.1 | 123.0 | +706.3% | 22.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1085 | 297.0 | 8226 | 1923 | 1376 | 1057 | +658.4% | 5.19 | **N better** |
| N vs P+ | write_p50_us | 437.7 | 45.8 | 938.7 | 47.4 | 46.6 | 159.3 | +114.4% | 10.8 | **N better** |
| N vs P+ | write_p99_us | 788.6 | 100.2 | 3079 | 378.8 | 277.1 | 591.2 | +290.5% | 8.27 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 2026 | 210.4 | 919.7 | 43.7 | 151.9 | 120.2 | -54.6% | 7.28 | **N better** |
| N vs P+ | pss_mib | 12.7 | 0.07 | 43.3 | 0.02 | 0.05 | 0.14 | +241.0% | 576.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 8204 | 281.6 | -17809 | 706.4 | 537.7 | 15490 | -317.1% | 48.4 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 107.6 | 20.6 | 102.4 | 14.6 | 17.8 | 0.67 | -4.8% | 0.29 | no difference |
| N vs P | q1_p99_us | 511.1 | 72.0 | 1062 | 147.9 | 116.4 | 280.3 | +107.7% | 4.73 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 131.5 | 19.1 | 211.3 | 7.53 | 14.5 | 20.9 | +60.6% | 5.49 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2126 | 131.3 | 123.2 | 16.4 | 93.6 | 177.1 | -94.2% | 21.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 117.7 | 21.2 | 105.0 | 32.5 | 27.5 | 19.3 | -10.8% | 0.46 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 654.7 | 293.3 | 2215 | 873.4 | 651.5 | 1195 | +238.3% | 2.40 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 147.8 | 29.5 | 218.3 | 19.9 | 25.1 | 47.6 | +47.6% | 2.80 | no difference |
| N vs P | c4_q1_p50_us | 165.4 | 24.0 | 175.6 | 51.1 | 39.9 | 132.4 | +6.2% | 0.26 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 905.3 | 216.1 | 2024 | 338.5 | 284.0 | 457.2 | +123.5% | 3.94 | **N better** |
| N vs P | c4_q2_p50_us | 223.8 | 21.9 | 300.9 | 34.1 | 28.6 | 135.8 | +34.4% | 2.69 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 1085 | 297.0 | 2636 | 468.3 | 392.1 | 275.4 | +143.0% | 3.96 | **N better** |
| N vs P | write_p50_us | 437.7 | 45.8 | 13328 | 502.0 | 356.4 | 916.6 | +2944.7% | 36.2 | **N better** |
| N vs P | write_p99_us | 788.6 | 100.2 | 19820 | 1959 | 1387 | 8495 | +2413.3% | 13.7 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | commits_per_s | 2026 | 210.4 | 71.5 | 2.91 | 148.8 | 50.6 | -96.5% | 13.1 | **N better** |
| N vs P | pss_mib | 12.7 | 0.07 | 43.2 | 0.01 | 0.05 | 0.13 | +240.6% | 591.6 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 8204 | 281.6 | -9669 | 590.4 | 462.5 | 9455 | -217.9% | 38.6 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 701.5 | 52.4 | 102.4 | 14.6 | 38.5 | 46.6 | -85.4% | 15.6 | **P better** |
| P+ vs P | q1_p99_us | 2534 | 572.1 | 1062 | 147.9 | 417.9 | 251.4 | -58.1% | 3.52 | **P better** |
| P+ vs P | q2_p50_us | 580.7 | 20.8 | 211.3 | 7.53 | 15.6 | 26.9 | -63.6% | 23.7 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 299.1 | 31.5 | 204.4 | 20.2 | 26.5 | 84.4 | -31.7% | 3.57 | **P better** |
| P+ vs P | q4_p50_us | 1352 | 52.3 | 208.6 | 19.7 | 39.5 | 27.8 | -84.6% | 28.9 | **P better** |
| P+ vs P | q5_p50_us | 3472 | 373.0 | 123.2 | 16.4 | 264.0 | 181.0 | -96.5% | 12.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 997.4 | 165.1 | 105.0 | 32.5 | 119.0 | 367.1 | -89.5% | 7.50 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q1_p99_us | 7926 | 2456 | 2215 | 873.4 | 1843 | 3793 | -72.1% | 3.10 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 817.0 | 115.6 | 218.3 | 19.9 | 82.9 | 318.3 | -73.3% | 7.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p50_us | 2042 | 122.1 | 175.6 | 51.1 | 93.6 | 93.3 | -91.4% | 19.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 9239 | 2655 | 2024 | 338.5 | 1893 | 1204 | -78.1% | 3.81 | **P better** |
| P+ vs P | c4_q2_p50_us | 1805 | 96.7 | 300.9 | 34.1 | 72.5 | 57.7 | -83.3% | 20.7 | **P better** |
| P+ vs P | c4_q2_p99_us | 8226 | 1923 | 2636 | 468.3 | 1399 | 1091 | -68.0% | 4.00 | **P better** |
| P+ vs P | write_p50_us | 938.7 | 47.4 | 13328 | 502.0 | 356.5 | 929.8 | +1319.8% | 34.7 | **P+ better** |
| P+ vs P | write_p99_us | 3079 | 378.8 | 19820 | 1959 | 1411 | 8484 | +543.7% | 11.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | commits_per_s | 919.7 | 43.7 | 71.5 | 2.91 | 30.9 | 109.8 | -92.2% | 27.4 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.02 | 43.2 | 0.01 | 0.02 | 0.05 | -0.1% | 3.41 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -17809 | 706.4 | -9669 | 590.4 | 651.0 | 16304 | -45.7% | 12.5 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 102.4 | 14.6 | 103.0 | 6.69 | 11.4 | 42.3 | +0.6% | 0.05 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q1_p99_us | 1062 | 147.9 | 1034 | 375.6 | 285.4 | 233.9 | -2.6% | 0.10 | BELOW FLOOR |
| P vs M | q2_p50_us | 211.3 | 7.53 | 203.3 | 26.8 | 19.7 | 38.2 | -3.8% | 0.41 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 204.4 | 20.2 | 227.1 | 34.3 | 28.2 | 78.6 | +11.1% | 0.81 | BELOW FLOOR |
| P vs M | q4_p50_us | 208.6 | 19.7 | 232.9 | 39.0 | 30.9 | 96.5 | +11.6% | 0.78 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q5_p50_us | 123.2 | 16.4 | 149.3 | 30.4 | 24.4 | 177.8 | +21.1% | 1.07 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 105.0 | 32.5 | 120.5 | 32.4 | 32.5 | 19.0 | +14.8% | 0.48 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 2215 | 873.4 | 1911 | 316.1 | 656.8 | 1225 | -13.7% | 0.46 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 218.3 | 19.9 | 218.3 | 50.3 | 38.3 | 49.0 | +0.0% | 0.00 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 175.6 | 51.1 | 167.0 | 39.0 | 45.4 | 112.8 | -4.9% | 0.19 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2024 | 338.5 | 2419 | 391.3 | 365.8 | 2396 | +19.5% | 1.08 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 300.9 | 34.1 | 287.6 | 42.2 | 38.4 | 90.5 | -4.4% | 0.35 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2636 | 468.3 | 3472 | 776.8 | 641.4 | 1246 | +31.7% | 1.30 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 13328 | 502.0 | 13321 | 552.5 | 527.8 | 949.3 | -0.1% | 0.01 | BELOW FLOOR |
| P vs M | write_p99_us | 19820 | 1959 | 19582 | 2560 | 2279 | 8661 | -1.2% | 0.10 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | commits_per_s | 71.5 | 2.91 | 71.5 | 3.67 | 3.32 | 9.12 | -0.0% | 0.00 | BELOW FLOOR |
| P vs M | pss_mib | 43.2 | 0.01 | 43.3 | 0.01 | 0.01 | 0.05 | +0.3% | 13.2 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -9669 | 590.4 | -6559 | 226.9 | 447.3 | 9228 | -32.2% | 6.95 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 107.6 | 20.6 | 299.7 | 32.8 | 27.4 | 153.1 | +178.6% | 7.02 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q1_p99_us | 511.1 | 72.0 | 838.9 | 296.1 | 215.5 | 453.4 | +64.1% | 1.52 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 131.5 | 19.1 | 301.2 | 25.3 | 22.4 | 60.1 | +129.0% | 7.57 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2126 | 131.3 | 3251 | 350.6 | 264.7 | 490.2 | +52.9% | 4.25 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 117.7 | 21.2 | 250.3 | 54.1 | 41.1 | 31.1 | +112.6% | 3.22 | **N better** |
| N vs H3 | c2_q1_p99_us | 654.7 | 293.3 | 1087 | 259.4 | 276.9 | 242.9 | +66.1% | 1.56 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 147.8 | 29.5 | 305.1 | 81.7 | 61.4 | 85.8 | +106.3% | 2.56 | no difference |
| N vs H3 | c4_q1_p50_us | 165.4 | 24.0 | 512.2 | 72.9 | 54.3 | 115.5 | +209.7% | 6.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 905.3 | 216.1 | 1775 | 309.9 | 267.2 | 600.4 | +96.1% | 3.26 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 223.8 | 21.9 | 543.7 | 94.8 | 68.8 | 135.3 | +142.9% | 4.65 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1085 | 297.0 | 1814 | 400.5 | 352.6 | 1165 | +67.3% | 2.07 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 437.7 | 45.8 | 1206 | 58.1 | 52.3 | 97.3 | +175.4% | 14.7 | **N better** |
| N vs H3 | write_p99_us | 788.6 | 100.2 | 4060 | 1075 | 763.4 | 6197 | +414.8% | 4.29 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 2026 | 210.4 | 697.2 | 76.1 | 158.2 | 127.7 | -65.6% | 8.40 | **N better** |
| N vs H3 | pss_mib | 12.7 | 0.07 | 67.1 | 0.33 | 0.24 | 0.15 | +429.0% | 228.8 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 8204 | 281.6 | 14902 | 275.3 | 278.5 | 12069 | +81.7% | 24.1 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 701.5 | 52.4 | 299.7 | 32.8 | 43.7 | 160.0 | -57.3% | 9.19 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p99_us | 2534 | 572.1 | 838.9 | 296.1 | 455.5 | 436.1 | -66.9% | 3.72 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 580.7 | 20.8 | 301.2 | 25.3 | 23.1 | 62.4 | -48.1% | 12.1 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 299.1 | 31.5 | 330.7 | 31.3 | 31.4 | 104.5 | +10.5% | 1.00 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1352 | 52.3 | 1460 | 188.1 | 138.0 | 114.3 | +7.9% | 0.78 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3472 | 373.0 | 3251 | 350.6 | 362.0 | 491.6 | -6.4% | 0.61 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 997.4 | 165.1 | 250.3 | 54.1 | 122.9 | 367.9 | -74.9% | 6.08 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q1_p99_us | 7926 | 2456 | 1087 | 259.4 | 1747 | 3609 | -86.3% | 3.92 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p50_us | 817.0 | 115.6 | 305.1 | 81.7 | 100.1 | 326.2 | -62.7% | 5.12 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 2042 | 122.1 | 512.2 | 72.9 | 100.6 | 67.3 | -74.9% | 15.2 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 9239 | 2655 | 1775 | 309.9 | 1890 | 1265 | -80.8% | 3.95 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1805 | 96.7 | 543.7 | 94.8 | 95.8 | 56.4 | -69.9% | 13.2 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8226 | 1923 | 1814 | 400.5 | 1389 | 1572 | -77.9% | 4.62 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 938.7 | 47.4 | 1206 | 58.1 | 53.0 | 183.9 | +28.4% | 5.03 | **P+ better** |
| P+ vs H3 | write_p99_us | 3079 | 378.8 | 4060 | 1075 | 806.0 | 6181 | +31.9% | 1.22 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 919.7 | 43.7 | 697.2 | 76.1 | 62.0 | 160.6 | -24.2% | 3.59 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.02 | 67.1 | 0.33 | 0.23 | 0.09 | +55.1% | 102.4 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -17809 | 706.4 | 14902 | 275.3 | 536.1 | 17947 | -183.7% | 61.0 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 107.6 | 20.6 | 1144 | 91.7 | 66.4 | 238.3 | +963.1% | 15.6 | **N better** |
| N vs T | q1_p99_us | 511.1 | 72.0 | 3458 | 556.6 | 396.8 | 310.2 | +576.6% | 7.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q2_p50_us | 131.5 | 19.1 | 1183 | 98.8 | 71.1 | 98.7 | +799.3% | 14.8 | **N better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 117.7 | 21.2 | 1718 | 68.2 | 50.5 | 524.6 | +1359.2% | 31.7 | **N better** |
| N vs T | c2_q1_p99_us | 654.7 | 293.3 | 5990 | 1799 | 1289 | 850.6 | +814.9% | 4.14 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q2_p50_us | 147.8 | 29.5 | 1765 | 57.2 | 45.5 | 695.2 | +1093.9% | 35.6 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c4_q1_p50_us | 165.4 | 24.0 | 2222 | 120.9 | 87.2 | 649.6 | +1243.2% | 23.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 905.3 | 216.1 | 4472 | 606.1 | 455.0 | 5521 | +394.0% | 7.84 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c4_q2_p50_us | 223.8 | 21.9 | 2252 | 161.9 | 115.5 | 499.9 | +906.4% | 17.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p99_us | 1085 | 297.0 | 4710 | 558.7 | 447.4 | 6831 | +334.3% | 8.10 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 437.7 | 45.8 | 1065 | 73.3 | 61.1 | 632.7 | +143.4% | 10.3 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p99_us | 788.6 | 100.2 | 2317 | 170.0 | 139.6 | 3332 | +193.8% | 11.0 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | commits_per_s | 2026 | 210.4 | 834.6 | 59.2 | 154.6 | 315.8 | -58.8% | 7.71 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | pss_mib | 12.7 | 0.07 | 2525 | 0.02 | 0.05 | 0.15 | +19795.6% | 46760 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 8204 | 281.6 | -5090 | 213.3 | 249.8 | 5653 | -162.0% | 53.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs T | q1_p50_us | 701.5 | 52.4 | 1144 | 91.7 | 74.7 | 242.8 | +63.0% | 5.92 | **P+ better** |
| P+ vs T | q1_p99_us | 2534 | 572.1 | 3458 | 556.6 | 564.4 | 284.4 | +36.4% | 1.64 | no difference |
| P+ vs T | q2_p50_us | 580.7 | 20.8 | 1183 | 98.8 | 71.4 | 100.2 | +103.7% | 8.44 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 997.4 | 165.1 | 1718 | 68.2 | 126.3 | 640.0 | +72.3% | 5.71 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c2_q1_p99_us | 7926 | 2456 | 5990 | 1799 | 2153 | 3699 | -24.4% | 0.90 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c2_q2_p50_us | 817.0 | 115.6 | 1765 | 57.2 | 91.2 | 763.1 | +116.1% | 10.4 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c4_q1_p50_us | 2042 | 122.1 | 2222 | 120.9 | 121.5 | 642.8 | +8.8% | 1.48 | BELOW FLOOR |
| P+ vs T | c4_q1_p99_us | 9239 | 2655 | 4472 | 606.1 | 1926 | 5632 | -51.6% | 2.48 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c4_q2_p50_us | 1805 | 96.7 | 2252 | 161.9 | 133.4 | 484.6 | +24.8% | 3.36 | BELOW FLOOR |
| P+ vs T | c4_q2_p99_us | 8226 | 1923 | 4710 | 558.7 | 1416 | 6912 | -42.7% | 2.48 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | write_p50_us | 938.7 | 47.4 | 1065 | 73.3 | 61.7 | 651.7 | +13.5% | 2.05 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | write_p99_us | 3079 | 378.8 | 2317 | 170.0 | 293.6 | 3304 | -24.8% | 2.60 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 919.7 | 43.7 | 834.6 | 59.2 | 52.0 | 330.5 | -9.3% | 1.64 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | pss_mib | 43.3 | 0.02 | 2525 | 0.02 | 0.02 | 0.09 | +5734.0% | 113847 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -17809 | 706.4 | -5090 | 213.3 | 521.8 | 14435 | -71.4% | 24.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| H3 vs T | q1_p50_us | 299.7 | 32.8 | 1144 | 91.7 | 68.8 | 283.2 | +281.6% | 12.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q1_p99_us | 838.9 | 296.1 | 3458 | 556.6 | 445.8 | 486.6 | +312.2% | 5.88 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q2_p50_us | 301.2 | 25.3 | 1183 | 98.8 | 72.1 | 115.6 | +292.7% | 12.2 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 250.3 | 54.1 | 1718 | 68.2 | 61.5 | 525.4 | +586.4% | 23.9 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 1087 | 259.4 | 5990 | 1799 | 1285 | 847.1 | +450.9% | 3.81 | **H3 better** |
| H3 vs T | c2_q2_p50_us | 305.1 | 81.7 | 1765 | 57.2 | 70.5 | 700.4 | +478.6% | 20.7 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c4_q1_p50_us | 512.2 | 72.9 | 2222 | 120.9 | 99.8 | 640.9 | +333.7% | 17.1 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 1775 | 309.9 | 4472 | 606.1 | 481.4 | 5548 | +151.9% | 5.60 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c4_q2_p50_us | 543.7 | 94.8 | 2252 | 161.9 | 132.7 | 487.8 | +314.3% | 12.9 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 1814 | 400.5 | 4710 | 558.7 | 486.1 | 6929 | +159.6% | 5.96 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | write_p50_us | 1206 | 58.1 | 1065 | 73.3 | 66.2 | 639.4 | -11.6% | 2.12 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | write_p99_us | 4060 | 1075 | 2317 | 170.0 | 769.6 | 6997 | -42.9% | 2.26 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | commits_per_s | 697.2 | 76.1 | 834.6 | 59.2 | 68.2 | 333.3 | +19.7% | 2.02 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | pss_mib | 67.1 | 0.33 | 2525 | 0.02 | 0.23 | 0.11 | +3661.2% | 10549 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 14902 | 275.3 | -5090 | 213.3 | 246.3 | 10682 | -134.2% | 81.2 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H1 | q1_p50_us | 107.6 | 20.6 | 844.0 | 24.5 | 22.6 | 41.9 | +684.6% | 32.6 | **N better** |
| N vs H1 | q1_p99_us | 511.1 | 72.0 | 2156 | 96.5 | 85.2 | 252.6 | +321.8% | 19.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p50_us | 131.5 | 19.1 | 702.4 | 23.7 | 21.5 | 29.1 | +434.1% | 26.5 | **N better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2126 | 131.3 | 447.1 | 34.0 | 95.9 | 100.7 | -79.0% | 17.5 | **H1 better** |
| N vs H1 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | c2_q1_p50_us | 117.7 | 21.2 | 875.6 | 29.9 | 26.0 | 128.8 | +643.7% | 29.2 | **N better** |
| N vs H1 | c2_q1_p99_us | 654.7 | 293.3 | 2681 | 401.5 | 351.6 | 413.3 | +309.6% | 5.76 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q2_p50_us | 147.8 | 29.5 | 818.1 | 47.7 | 39.7 | 147.8 | +453.3% | 16.9 | **N better** |
| N vs H1 | c4_q1_p50_us | 165.4 | 24.0 | 854.8 | 38.2 | 31.9 | 132.2 | +416.8% | 21.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 905.3 | 216.1 | 2152 | 166.2 | 192.8 | 930.2 | +137.8% | 6.47 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c4_q2_p50_us | 223.8 | 21.9 | 814.9 | 43.5 | 34.4 | 126.3 | +264.1% | 17.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p99_us | 1085 | 297.0 | 3301 | 886.2 | 660.9 | 249.1 | +204.3% | 3.35 | **N better** |
| N vs H1 | write_p50_us | 437.7 | 45.8 | 1455 | 60.3 | 53.5 | 272.1 | +232.4% | 19.0 | **N better** |
| N vs H1 | write_p99_us | 788.6 | 100.2 | 5207 | 1221 | 866.1 | 2869 | +560.3% | 5.10 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | commits_per_s | 2026 | 210.4 | 555.9 | 55.2 | 153.8 | 70.7 | -72.6% | 9.56 | **N better** |
| N vs H1 | pss_mib | 12.7 | 0.07 | 285.6 | 5.83 | 4.12 | 7.14 | +2150.9% | 66.2 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 8204 | 281.6 | 31531 | 4910 | 3478 | 6739 | +284.4% | 6.71 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H1 | q1_p50_us | 701.5 | 52.4 | 844.0 | 24.5 | 40.9 | 62.6 | +20.3% | 3.48 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2534 | 572.1 | 2156 | 96.5 | 410.3 | 220.1 | -14.9% | 0.92 | no difference |
| P+ vs H1 | q2_p50_us | 580.7 | 20.8 | 702.4 | 23.7 | 22.3 | 33.7 | +21.0% | 5.46 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 299.1 | 31.5 | 780.9 | 30.7 | 31.1 | 211.2 | +161.0% | 15.5 | **P+ better** |
| P+ vs H1 | q4_p50_us | 1352 | 52.3 | 2965 | 225.2 | 163.5 | 103.9 | +119.3% | 9.86 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3472 | 373.0 | 447.1 | 34.0 | 264.9 | 107.4 | -87.1% | 11.4 | **H1 better** |
| P+ vs H1 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | c2_q1_p50_us | 997.4 | 165.1 | 875.6 | 29.9 | 118.7 | 388.5 | -12.2% | 1.03 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c2_q1_p99_us | 7926 | 2456 | 2681 | 401.5 | 1760 | 3624 | -66.2% | 2.98 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c2_q2_p50_us | 817.0 | 115.6 | 818.1 | 47.7 | 88.4 | 347.7 | +0.1% | 0.01 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q1_p50_us | 2042 | 122.1 | 854.8 | 38.2 | 90.5 | 93.0 | -58.1% | 13.1 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 9239 | 2655 | 2152 | 166.2 | 1881 | 1451 | -76.7% | 3.77 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | c4_q2_p50_us | 1805 | 96.7 | 814.9 | 43.5 | 75.0 | 28.9 | -54.8% | 13.2 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 8226 | 1923 | 3301 | 886.2 | 1497 | 1085 | -59.9% | 3.29 | **H1 better** |
| P+ vs H1 | write_p50_us | 938.7 | 47.4 | 1455 | 60.3 | 54.3 | 313.7 | +55.0% | 9.52 | **P+ better** |
| P+ vs H1 | write_p99_us | 3079 | 378.8 | 5207 | 1221 | 903.8 | 2835 | +69.1% | 2.35 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | commits_per_s | 919.7 | 43.7 | 555.9 | 55.2 | 49.8 | 120.4 | -39.6% | 7.31 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.02 | 285.6 | 5.83 | 4.12 | 7.14 | +560.0% | 58.8 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -17809 | 706.4 | 31531 | 4910 | 3508 | 14894 | -277.1% | 14.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 107.6 | 20.6 | 83.6 | 10.7 | 16.4 | 9.69 | -22.3% | 1.46 | no difference |
| N vs H2 | q1_p99_us | 511.1 | 72.0 | 518.6 | 191.1 | 144.4 | 436.3 | +1.5% | 0.05 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | q2_p50_us | 131.5 | 19.1 | 176.9 | 31.5 | 26.0 | 53.5 | +34.5% | 1.74 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2126 | 131.3 | 128.4 | 21.4 | 94.1 | 54.5 | -94.0% | 21.2 | **H2 better** |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 117.7 | 21.2 | 110.9 | 29.5 | 25.7 | 17.1 | -5.8% | 0.26 | BELOW FLOOR |
| N vs H2 | c2_q1_p99_us | 654.7 | 293.3 | 1874 | 523.5 | 424.3 | 469.7 | +186.2% | 2.87 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q2_p50_us | 147.8 | 29.5 | 228.5 | 24.1 | 26.9 | 58.3 | +54.6% | 3.00 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p50_us | 165.4 | 24.0 | 118.5 | 15.3 | 20.1 | 127.3 | -28.3% | 2.33 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q1_p99_us | 905.3 | 216.1 | 1757 | 322.9 | 274.8 | 672.1 | +94.1% | 3.10 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p50_us | 223.8 | 21.9 | 239.7 | 16.1 | 19.2 | 141.4 | +7.1% | 0.83 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p99_us | 1085 | 297.0 | 2302 | 488.7 | 404.3 | 100.6 | +112.2% | 3.01 | **N better** |
| N vs H2 | write_p50_us | 437.7 | 45.8 | 1309 | 109.9 | 84.2 | 38.6 | +199.1% | 10.3 | **N better** |
| N vs H2 | write_p99_us | 788.6 | 100.2 | 4832 | 485.7 | 350.7 | 1247 | +512.8% | 11.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 2026 | 210.4 | 642.8 | 60.5 | 154.8 | 55.2 | -68.3% | 8.94 | **N better** |
| N vs H2 | pss_mib | 12.7 | 0.07 | 43.5 | 0.01 | 0.05 | 0.14 | +242.9% | 599.2 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 8204 | 281.6 | -3437 | 133.5 | 220.3 | 6279 | -141.9% | 52.8 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 701.5 | 52.4 | 83.6 | 10.7 | 37.8 | 47.6 | -88.1% | 16.3 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2534 | 572.1 | 518.6 | 191.1 | 426.5 | 418.3 | -79.5% | 4.73 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p50_us | 580.7 | 20.8 | 176.9 | 31.5 | 26.6 | 56.1 | -69.5% | 15.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 299.1 | 31.5 | 204.3 | 17.9 | 25.6 | 70.4 | -31.7% | 3.70 | **H2 better** |
| P+ vs H2 | q4_p50_us | 1352 | 52.3 | 192.1 | 23.6 | 40.6 | 9.20 | -85.8% | 28.6 | **H2 better** |
| P+ vs H2 | q5_p50_us | 3472 | 373.0 | 128.4 | 21.4 | 264.2 | 66.1 | -96.3% | 12.7 | **H2 better** |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 997.4 | 165.1 | 110.9 | 29.5 | 118.6 | 367.0 | -88.9% | 7.47 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q1_p99_us | 7926 | 2456 | 1874 | 523.5 | 1776 | 3631 | -76.4% | 3.41 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q2_p50_us | 817.0 | 115.6 | 228.5 | 24.1 | 83.5 | 320.1 | -72.0% | 7.05 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c4_q1_p50_us | 2042 | 122.1 | 118.5 | 15.3 | 87.0 | 85.9 | -94.2% | 22.1 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 9239 | 2655 | 1757 | 322.9 | 1892 | 1301 | -81.0% | 3.96 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p50_us | 1805 | 96.7 | 239.7 | 16.1 | 69.3 | 69.8 | -86.7% | 22.6 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 8226 | 1923 | 2302 | 488.7 | 1403 | 1060 | -72.0% | 4.22 | **H2 better** |
| P+ vs H2 | write_p50_us | 938.7 | 47.4 | 1309 | 109.9 | 84.7 | 160.8 | +39.5% | 4.38 | **P+ better** |
| P+ vs H2 | write_p99_us | 3079 | 378.8 | 4832 | 485.7 | 435.6 | 1168 | +56.9% | 4.03 | **P+ better** |
| P+ vs H2 | commits_per_s | 919.7 | 43.7 | 642.8 | 60.5 | 52.8 | 112.0 | -30.1% | 5.25 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.02 | 43.5 | 0.01 | 0.02 | 0.06 | +0.5% | 15.3 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -17809 | 706.4 | -3437 | 133.5 | 508.4 | 14692 | -80.7% | 28.3 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 109.9 | 9.58 | 695.5 | 54.9 | 39.4 | 51.9 | +532.7% | 14.9 | **N better** |
| N vs P+ | q1_p99_us | 501.5 | 173.1 | 2571 | 1013 | 726.4 | 1728 | +412.7% | 2.85 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 132.4 | 16.2 | 528.6 | 38.9 | 29.8 | 90.7 | +299.2% | 13.3 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2248 | 131.8 | 3145 | 128.1 | 130.0 | 204.1 | +39.9% | 6.90 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 127.5 | 19.6 | 777.7 | 57.7 | 43.1 | 36.4 | +510.0% | 15.1 | **N better** |
| N vs P+ | c2_q1_p99_us | 637.7 | 235.2 | 5271 | 674.7 | 505.2 | 767.2 | +726.6% | 9.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 147.2 | 13.2 | 634.8 | 45.2 | 33.3 | 52.9 | +331.2% | 14.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 232.8 | 24.3 | 2097 | 170.0 | 121.4 | 294.1 | +800.6% | 15.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1018 | 256.5 | 11098 | 2290 | 1630 | 3515 | +990.4% | 6.19 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 247.0 | 36.9 | 1732 | 98.3 | 74.3 | 405.1 | +601.5% | 20.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 871.6 | 142.7 | 8468 | 3248 | 2299 | 2046 | +871.6% | 3.30 | **N better** |
| N vs P+ | write_p50_us | 422.5 | 32.4 | 946.6 | 62.4 | 49.7 | 136.5 | +124.0% | 10.5 | **N better** |
| N vs P+ | write_p99_us | 1107 | 292.0 | 2780 | 288.4 | 290.2 | 689.7 | +151.1% | 5.77 | **N better** |
| N vs P+ | commits_per_s | 2030 | 190.8 | 889.2 | 36.7 | 137.4 | 441.6 | -56.2% | 8.30 | **N better** |
| N vs P+ | pss_mib | 13.5 | 0.47 | 43.3 | 0.01 | 0.33 | 0.28 | +219.8% | 89.4 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 225.8 | 492.2 | -2597 | 123.9 | 358.9 | 2510 | -1250.4% | 7.87 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 109.9 | 9.58 | 110.9 | 16.4 | 13.4 | 27.6 | +0.9% | 0.07 | BELOW FLOOR |
| N vs P | q1_p99_us | 501.5 | 173.1 | 860.7 | 282.7 | 234.4 | 593.8 | +71.6% | 1.53 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 132.4 | 16.2 | 194.0 | 26.9 | 22.2 | 31.8 | +46.5% | 2.77 | no difference |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2248 | 131.8 | 151.5 | 35.2 | 96.5 | 86.7 | -93.3% | 21.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 127.5 | 19.6 | 131.6 | 21.1 | 20.4 | 87.3 | +3.2% | 0.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 637.7 | 235.2 | 1926 | 344.8 | 295.2 | 979.7 | +202.0% | 4.36 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 147.2 | 13.2 | 223.4 | 19.7 | 16.8 | 156.2 | +51.8% | 4.54 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p50_us | 232.8 | 24.3 | 142.2 | 12.5 | 19.3 | 189.5 | -38.9% | 4.69 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1018 | 256.5 | 2129 | 106.6 | 196.4 | 745.9 | +109.2% | 5.66 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 247.0 | 36.9 | 291.6 | 28.3 | 32.9 | 192.6 | +18.1% | 1.36 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 871.6 | 142.7 | 3702 | 583.0 | 424.4 | 800.7 | +324.7% | 6.67 | **N better** |
| N vs P | write_p50_us | 422.5 | 32.4 | 13580 | 379.5 | 269.3 | 131.8 | +3114.2% | 48.9 | **N better** |
| N vs P | write_p99_us | 1107 | 292.0 | 20774 | 4396 | 3115 | 790.9 | +1776.4% | 6.31 | **N better** |
| N vs P | commits_per_s | 2030 | 190.8 | 69.3 | 3.78 | 134.9 | 441.5 | -96.6% | 14.5 | **N better** |
| N vs P | pss_mib | 13.5 | 0.47 | 43.3 | 0.01 | 0.33 | 0.29 | +220.1% | 89.5 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 225.8 | 492.2 | -2551 | 103.5 | 355.7 | 2492 | -1230.0% | 7.81 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 695.5 | 54.9 | 110.9 | 16.4 | 40.5 | 44.6 | -84.1% | 14.4 | **P better** |
| P+ vs P | q1_p99_us | 2571 | 1013 | 860.7 | 282.7 | 743.3 | 1718 | -66.5% | 2.30 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 528.6 | 38.9 | 194.0 | 26.9 | 33.4 | 89.3 | -63.3% | 10.0 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 278.1 | 32.7 | 235.3 | 51.7 | 43.2 | 78.6 | -15.4% | 0.99 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1409 | 101.5 | 205.8 | 26.1 | 74.1 | 486.3 | -85.4% | 16.2 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q5_p50_us | 3145 | 128.1 | 151.5 | 35.2 | 93.9 | 215.1 | -95.2% | 31.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 777.7 | 57.7 | 131.6 | 21.1 | 43.4 | 88.4 | -83.1% | 14.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5271 | 674.7 | 1926 | 344.8 | 535.8 | 1057 | -63.5% | 6.24 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 634.8 | 45.2 | 223.4 | 19.7 | 34.9 | 146.9 | -64.8% | 11.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 2097 | 170.0 | 142.2 | 12.5 | 120.5 | 250.4 | -93.2% | 16.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 11098 | 2290 | 2129 | 106.6 | 1621 | 3503 | -80.8% | 5.53 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1732 | 98.3 | 291.6 | 28.3 | 72.4 | 370.4 | -83.2% | 19.9 | **P better** |
| P+ vs P | c4_q2_p99_us | 8468 | 3248 | 3702 | 583.0 | 2333 | 2179 | -56.3% | 2.04 | no difference |
| P+ vs P | write_p50_us | 946.6 | 62.4 | 13580 | 379.5 | 271.9 | 39.0 | +1334.7% | 46.5 | **P+ better** |
| P+ vs P | write_p99_us | 2780 | 288.4 | 20774 | 4396 | 3115 | 1003 | +647.2% | 5.78 | **P+ better** |
| P+ vs P | commits_per_s | 889.2 | 36.7 | 69.3 | 3.78 | 26.1 | 6.97 | -92.2% | 31.4 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.01 | 43.3 | 0.01 | 0.01 | 0.06 | +0.1% | 3.48 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2597 | 123.9 | -2551 | 103.5 | 114.1 | 2756 | -1.8% | 0.40 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 110.9 | 16.4 | 117.7 | 13.9 | 15.2 | 8.52 | +6.1% | 0.44 | BELOW FLOOR |
| P vs M | q1_p99_us | 860.7 | 282.7 | 905.3 | 172.7 | 234.2 | 504.8 | +5.2% | 0.19 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 194.0 | 26.9 | 197.6 | 10.3 | 20.4 | 69.1 | +1.8% | 0.18 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 235.3 | 51.7 | 226.5 | 37.0 | 44.9 | 60.6 | -3.8% | 0.20 | BELOW FLOOR |
| P vs M | q4_p50_us | 205.8 | 26.1 | 221.2 | 43.5 | 35.9 | 88.8 | +7.5% | 0.43 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q5_p50_us | 151.5 | 35.2 | 160.7 | 49.9 | 43.2 | 86.2 | +6.1% | 0.21 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 131.6 | 21.1 | 104.8 | 30.9 | 26.5 | 88.9 | -20.4% | 1.01 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 1926 | 344.8 | 1675 | 423.8 | 386.4 | 866.1 | -13.0% | 0.65 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 223.4 | 19.7 | 180.6 | 26.2 | 23.2 | 147.0 | -19.2% | 1.85 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 142.2 | 12.5 | 137.9 | 44.1 | 32.4 | 98.5 | -3.0% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2129 | 106.6 | 1986 | 321.8 | 239.7 | 668.3 | -6.7% | 0.60 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 291.6 | 28.3 | 235.5 | 35.6 | 32.2 | 77.2 | -19.3% | 1.74 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3702 | 583.0 | 2825 | 998.0 | 817.3 | 774.7 | -23.7% | 1.07 | no difference |
| P vs M | write_p50_us | 13580 | 379.5 | 14088 | 513.6 | 451.6 | 66.3 | +3.7% | 1.12 | no difference |
| P vs M | write_p99_us | 20774 | 4396 | 20744 | 2101 | 3445 | 6459 | -0.1% | 0.01 | BELOW FLOOR |
| P vs M | commits_per_s | 69.3 | 3.78 | 70.5 | 4.59 | 4.20 | 9.14 | +1.7% | 0.28 | BELOW FLOOR |
| P vs M | pss_mib | 43.3 | 0.01 | 43.3 | 0.01 | 0.01 | 0.07 | +0.0% | 1.30 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2551 | 103.5 | -2497 | 131.2 | 118.2 | 2566 | -2.1% | 0.46 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 109.9 | 9.58 | 343.2 | 31.0 | 23.0 | 28.3 | +212.2% | 10.2 | **N better** |
| N vs H3 | q1_p99_us | 501.5 | 173.1 | 958.0 | 164.9 | 169.1 | 978.0 | +91.0% | 2.70 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 132.4 | 16.2 | 349.1 | 42.3 | 32.0 | 25.3 | +163.7% | 6.77 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2248 | 131.8 | 3499 | 264.3 | 208.8 | 323.6 | +55.6% | 5.99 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 127.5 | 19.6 | 282.4 | 42.0 | 32.8 | 75.8 | +121.5% | 4.73 | **N better** |
| N vs H3 | c2_q1_p99_us | 637.7 | 235.2 | 1101 | 233.8 | 234.5 | 671.5 | +72.6% | 1.97 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 147.2 | 13.2 | 311.5 | 55.9 | 40.6 | 176.5 | +111.6% | 4.04 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p50_us | 232.8 | 24.3 | 513.7 | 58.0 | 44.5 | 173.5 | +120.6% | 6.32 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 1018 | 256.5 | 1889 | 409.0 | 341.4 | 752.9 | +85.6% | 2.55 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 247.0 | 36.9 | 531.2 | 67.0 | 54.1 | 290.5 | +115.1% | 5.26 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p99_us | 871.6 | 142.7 | 2089 | 529.2 | 387.6 | 332.2 | +139.7% | 3.14 | **N better** |
| N vs H3 | write_p50_us | 422.5 | 32.4 | 1344 | 65.9 | 52.0 | 148.7 | +218.0% | 17.7 | **N better** |
| N vs H3 | write_p99_us | 1107 | 292.0 | 3862 | 705.1 | 539.7 | 437.6 | +248.9% | 5.11 | **N better** |
| N vs H3 | commits_per_s | 2030 | 190.8 | 645.6 | 33.2 | 136.9 | 461.4 | -68.2% | 10.1 | **N better** |
| N vs H3 | pss_mib | 13.5 | 0.47 | 67.3 | 0.39 | 0.43 | 0.37 | +397.4% | 124.5 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 225.8 | 492.2 | 16221 | 364.9 | 433.3 | 9079 | +7084.3% | 36.9 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 695.5 | 54.9 | 343.2 | 31.0 | 44.6 | 45.1 | -50.7% | 7.90 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2571 | 1013 | 958.0 | 164.9 | 725.4 | 1886 | -62.7% | 2.22 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 528.6 | 38.9 | 349.1 | 42.3 | 40.6 | 87.2 | -34.0% | 4.42 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 278.1 | 32.7 | 346.2 | 26.9 | 30.0 | 80.8 | +24.5% | 2.27 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1409 | 101.5 | 1549 | 96.8 | 99.2 | 493.6 | +10.0% | 1.41 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q5_p50_us | 3145 | 128.1 | 3499 | 264.3 | 207.7 | 378.8 | +11.3% | 1.70 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 777.7 | 57.7 | 282.4 | 42.0 | 50.4 | 77.1 | -63.7% | 9.82 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5271 | 674.7 | 1101 | 233.8 | 504.9 | 779.8 | -79.1% | 8.26 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 634.8 | 45.2 | 311.5 | 55.9 | 50.9 | 168.3 | -50.9% | 6.36 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2097 | 170.0 | 513.7 | 58.0 | 127.0 | 238.6 | -75.5% | 12.5 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 11098 | 2290 | 1889 | 409.0 | 1645 | 3505 | -83.0% | 5.60 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1732 | 98.3 | 531.2 | 67.0 | 84.1 | 429.6 | -69.3% | 14.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 8468 | 3248 | 2089 | 529.2 | 2327 | 2053 | -75.3% | 2.74 | no difference |
| P+ vs H3 | write_p50_us | 946.6 | 62.4 | 1344 | 65.9 | 64.2 | 79.1 | +41.9% | 6.19 | **P+ better** |
| P+ vs H3 | write_p99_us | 2780 | 288.4 | 3862 | 705.1 | 538.7 | 755.8 | +38.9% | 2.01 | no difference |
| P+ vs H3 | commits_per_s | 889.2 | 36.7 | 645.6 | 33.2 | 35.0 | 134.1 | -27.4% | 6.96 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.01 | 67.3 | 0.39 | 0.28 | 0.25 | +55.5% | 87.2 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2597 | 123.9 | 16221 | 364.9 | 272.5 | 9155 | -724.5% | 69.1 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 109.9 | 9.58 | 1135 | 66.0 | 47.1 | 87.5 | +932.6% | 21.8 | **N better** |
| N vs T | q1_p99_us | 501.5 | 173.1 | 2734 | 208.4 | 191.6 | 1162 | +445.2% | 11.7 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | q2_p50_us | 132.4 | 16.2 | 1140 | 32.0 | 25.3 | 62.6 | +761.2% | 39.8 | **N better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 127.5 | 19.6 | 1752 | 97.6 | 70.4 | 187.9 | +1274.4% | 23.1 | **N better** |
| N vs T | c2_q1_p99_us | 637.7 | 235.2 | 4849 | 1867 | 1331 | 487.3 | +660.3% | 3.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q2_p50_us | 147.2 | 13.2 | 1775 | 143.8 | 102.1 | 232.8 | +1105.6% | 15.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p50_us | 232.8 | 24.3 | 2172 | 155.1 | 111.0 | 196.1 | +833.0% | 17.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 1018 | 256.5 | 5367 | 1878 | 1341 | 1315 | +427.3% | 3.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p50_us | 247.0 | 36.9 | 2195 | 189.4 | 136.4 | 342.5 | +788.7% | 14.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p99_us | 871.6 | 142.7 | 5614 | 1759 | 1248 | 750.4 | +544.1% | 3.80 | **N better** |
| N vs T | write_p50_us | 422.5 | 32.4 | 1046 | 61.7 | 49.3 | 133.3 | +147.7% | 12.7 | **N better** |
| N vs T | write_p99_us | 1107 | 292.0 | 2155 | 480.5 | 397.6 | 1032 | +94.6% | 2.63 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | commits_per_s | 2030 | 190.8 | 854.3 | 71.4 | 144.0 | 493.3 | -57.9% | 8.16 | **N better** |
| N vs T | pss_mib | 13.5 | 0.47 | 2524 | 0.01 | 0.33 | 0.30 | +18547.5% | 7541 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 225.8 | 492.2 | -4034 | 187.6 | 372.5 | 3583 | -1886.9% | 11.4 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 695.5 | 54.9 | 1135 | 66.0 | 60.7 | 94.3 | +63.2% | 7.24 | **P+ better** |
| P+ vs T | q1_p99_us | 2571 | 1013 | 2734 | 208.4 | 731.0 | 1987 | +6.3% | 0.22 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | q2_p50_us | 528.6 | 38.9 | 1140 | 32.0 | 35.6 | 104.3 | +115.7% | 17.2 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 777.7 | 57.7 | 1752 | 97.6 | 80.2 | 188.4 | +125.3% | 12.2 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 5271 | 674.7 | 4849 | 1867 | 1404 | 628.3 | -8.0% | 0.30 | BELOW FLOOR |
| P+ vs T | c2_q2_p50_us | 634.8 | 45.2 | 1775 | 143.8 | 106.6 | 226.7 | +179.6% | 10.7 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2097 | 170.0 | 2172 | 155.1 | 162.7 | 255.4 | +3.6% | 0.46 | BELOW FLOOR |
| P+ vs T | c4_q1_p99_us | 11098 | 2290 | 5367 | 1878 | 2095 | 3667 | -51.6% | 2.74 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q2_p50_us | 1732 | 98.3 | 2195 | 189.4 | 150.9 | 466.3 | +26.7% | 3.06 | BELOW FLOOR |
| P+ vs T | c4_q2_p99_us | 8468 | 3248 | 5614 | 1759 | 2612 | 2161 | -33.7% | 1.09 | no difference |
| P+ vs T | write_p50_us | 946.6 | 62.4 | 1046 | 61.7 | 62.1 | 43.7 | +10.6% | 1.61 | no difference |
| P+ vs T | write_p99_us | 2780 | 288.4 | 2155 | 480.5 | 396.3 | 1202 | -22.5% | 1.58 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 889.2 | 36.7 | 854.3 | 71.4 | 56.8 | 220.1 | -3.9% | 0.62 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.01 | 2524 | 0.01 | 0.01 | 0.11 | +5730.2% | 239528 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -2597 | 123.9 | -4034 | 187.6 | 159.0 | 3771 | +55.3% | 9.04 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 343.2 | 31.0 | 1135 | 66.0 | 51.6 | 83.6 | +230.8% | 15.4 | **H3 better** |
| H3 vs T | q1_p99_us | 958.0 | 164.9 | 2734 | 208.4 | 187.9 | 1385 | +185.4% | 9.45 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | q2_p50_us | 349.1 | 42.3 | 1140 | 32.0 | 37.5 | 57.3 | +226.6% | 21.1 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 282.4 | 42.0 | 1752 | 97.6 | 75.2 | 199.8 | +520.5% | 19.6 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 1101 | 233.8 | 4849 | 1867 | 1331 | 507.1 | +340.6% | 2.82 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q2_p50_us | 311.5 | 55.9 | 1775 | 143.8 | 109.1 | 282.4 | +469.7% | 13.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p50_us | 513.7 | 58.0 | 2172 | 155.1 | 117.1 | 94.1 | +322.8% | 14.2 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 1889 | 409.0 | 5367 | 1878 | 1359 | 1288 | +184.1% | 2.56 | no difference |
| H3 vs T | c4_q2_p50_us | 531.2 | 67.0 | 2195 | 189.4 | 142.0 | 371.2 | +313.2% | 11.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p99_us | 2089 | 529.2 | 5614 | 1759 | 1299 | 769.1 | +168.8% | 2.71 | no difference |
| H3 vs T | write_p50_us | 1344 | 65.9 | 1046 | 61.7 | 63.9 | 73.3 | -22.1% | 4.65 | **T better** |
| H3 vs T | write_p99_us | 3862 | 705.1 | 2155 | 480.5 | 603.4 | 1077 | -44.2% | 2.83 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 645.6 | 33.2 | 854.3 | 71.4 | 55.7 | 257.6 | +32.3% | 3.75 | BELOW FLOOR |
| H3 vs T | pss_mib | 67.3 | 0.39 | 2524 | 0.01 | 0.28 | 0.27 | +3649.1% | 8914 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 16221 | 364.9 | -4034 | 187.6 | 290.1 | 9505 | -124.9% | 69.8 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 109.9 | 9.58 | 868.5 | 46.0 | 33.2 | 27.2 | +690.0% | 22.8 | **N better** |
| N vs H1 | q1_p99_us | 501.5 | 173.1 | 2396 | 453.8 | 343.4 | 461.0 | +377.7% | 5.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p50_us | 132.4 | 16.2 | 728.2 | 40.8 | 31.0 | 71.2 | +450.0% | 19.2 | **N better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2248 | 131.8 | 530.3 | 77.1 | 108.0 | 38.9 | -76.4% | 15.9 | **H1 better** |
| N vs H1 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | c2_q1_p50_us | 127.5 | 19.6 | 955.8 | 32.1 | 26.6 | 56.7 | +649.6% | 31.1 | **N better** |
| N vs H1 | c2_q1_p99_us | 637.7 | 235.2 | 2659 | 402.7 | 329.8 | 465.3 | +317.0% | 6.13 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q2_p50_us | 147.2 | 13.2 | 869.3 | 8.15 | 11.0 | 116.3 | +490.6% | 65.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p50_us | 232.8 | 24.3 | 877.5 | 67.7 | 50.8 | 172.8 | +276.9% | 12.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 1018 | 256.5 | 2393 | 253.6 | 255.0 | 581.3 | +135.1% | 5.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p50_us | 247.0 | 36.9 | 802.8 | 46.9 | 42.2 | 187.4 | +225.1% | 13.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p99_us | 871.6 | 142.7 | 2852 | 468.2 | 346.1 | 380.4 | +227.2% | 5.72 | **N better** |
| N vs H1 | write_p50_us | 422.5 | 32.4 | 1502 | 60.6 | 48.6 | 198.8 | +255.5% | 22.2 | **N better** |
| N vs H1 | write_p99_us | 1107 | 292.0 | 5073 | 699.8 | 536.1 | 1739 | +358.2% | 7.40 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | commits_per_s | 2030 | 190.8 | 580.1 | 24.1 | 136.0 | 453.3 | -71.4% | 10.7 | **N better** |
| N vs H1 | pss_mib | 13.5 | 0.47 | 287.2 | 5.93 | 4.21 | 10.4 | +2021.2% | 65.0 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 225.8 | 492.2 | 37920 | 5382 | 3822 | 5281 | +16694.8% | 9.86 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H1 | q1_p50_us | 695.5 | 54.9 | 868.5 | 46.0 | 50.7 | 44.3 | +24.9% | 3.41 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2571 | 1013 | 2396 | 453.8 | 784.6 | 1677 | -6.8% | 0.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q2_p50_us | 528.6 | 38.9 | 728.2 | 40.8 | 39.8 | 109.7 | +37.8% | 5.01 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 278.1 | 32.7 | 766.3 | 50.0 | 42.3 | 240.7 | +175.5% | 11.5 | **P+ better** |
| P+ vs H1 | q4_p50_us | 1409 | 101.5 | 3054 | 240.8 | 184.8 | 573.0 | +116.8% | 8.90 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q5_p50_us | 3145 | 128.1 | 530.3 | 77.1 | 105.7 | 200.7 | -83.1% | 24.7 | **H1 better** |
| P+ vs H1 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | c2_q1_p50_us | 777.7 | 57.7 | 955.8 | 32.1 | 46.7 | 58.4 | +22.9% | 3.82 | **P+ better** |
| P+ vs H1 | c2_q1_p99_us | 5271 | 674.7 | 2659 | 402.7 | 555.6 | 611.4 | -49.6% | 4.70 | **H1 better** |
| P+ vs H1 | c2_q2_p50_us | 634.8 | 45.2 | 869.3 | 8.15 | 32.5 | 103.6 | +37.0% | 7.22 | **P+ better** |
| P+ vs H1 | c4_q1_p50_us | 2097 | 170.0 | 877.5 | 67.7 | 129.4 | 238.1 | -58.1% | 9.42 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 11098 | 2290 | 2393 | 253.6 | 1629 | 3472 | -78.4% | 5.34 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q2_p50_us | 1732 | 98.3 | 802.8 | 46.9 | 77.0 | 367.7 | -53.7% | 12.1 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 8468 | 3248 | 2852 | 468.2 | 2320 | 2062 | -66.3% | 2.42 | no difference |
| P+ vs H1 | write_p50_us | 946.6 | 62.4 | 1502 | 60.6 | 61.5 | 153.8 | +58.7% | 9.03 | **P+ better** |
| P+ vs H1 | write_p99_us | 2780 | 288.4 | 5073 | 699.8 | 535.2 | 1845 | +82.5% | 4.28 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | commits_per_s | 889.2 | 36.7 | 580.1 | 24.1 | 31.0 | 103.0 | -34.8% | 9.96 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.01 | 287.2 | 5.93 | 4.19 | 10.4 | +563.2% | 58.1 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -2597 | 123.9 | 37920 | 5382 | 3807 | 5411 | -1559.9% | 10.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 109.9 | 9.58 | 98.8 | 6.13 | 8.04 | 27.1 | -10.1% | 1.38 | BELOW FLOOR |
| N vs H2 | q1_p99_us | 501.5 | 173.1 | 756.4 | 395.3 | 305.1 | 609.9 | +50.8% | 0.84 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | q2_p50_us | 132.4 | 16.2 | 192.4 | 27.3 | 22.5 | 29.2 | +45.3% | 2.67 | no difference |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2248 | 131.8 | 143.1 | 30.2 | 95.6 | 40.1 | -93.6% | 22.0 | **H2 better** |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 127.5 | 19.6 | 87.6 | 52.1 | 39.4 | 29.4 | -31.3% | 1.01 | no difference |
| N vs H2 | c2_q1_p99_us | 637.7 | 235.2 | 1726 | 321.0 | 281.4 | 886.0 | +170.7% | 3.87 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 147.2 | 13.2 | 147.5 | 43.9 | 32.4 | 140.1 | +0.2% | 0.01 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q1_p50_us | 232.8 | 24.3 | 148.6 | 27.4 | 25.9 | 173.5 | -36.2% | 3.25 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p99_us | 1018 | 256.5 | 2175 | 390.0 | 330.0 | 668.7 | +113.7% | 3.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 247.0 | 36.9 | 291.6 | 37.1 | 37.0 | 186.5 | +18.1% | 1.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p99_us | 871.6 | 142.7 | 3160 | 943.7 | 674.9 | 312.2 | +262.5% | 3.39 | **N better** |
| N vs H2 | write_p50_us | 422.5 | 32.4 | 1372 | 43.0 | 38.1 | 131.4 | +224.7% | 24.9 | **N better** |
| N vs H2 | write_p99_us | 1107 | 292.0 | 4902 | 386.5 | 342.5 | 305.1 | +342.8% | 11.1 | **N better** |
| N vs H2 | commits_per_s | 2030 | 190.8 | 617.0 | 26.2 | 136.2 | 441.6 | -69.6% | 10.4 | **N better** |
| N vs H2 | pss_mib | 13.5 | 0.47 | 43.5 | 0.01 | 0.33 | 0.29 | +221.4% | 90.0 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 225.8 | 492.2 | -2672 | 118.2 | 358.0 | 2576 | -1283.5% | 8.10 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 695.5 | 54.9 | 98.8 | 6.13 | 39.1 | 44.3 | -85.8% | 15.3 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2571 | 1013 | 756.4 | 395.3 | 768.6 | 1724 | -70.6% | 2.36 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q2_p50_us | 528.6 | 38.9 | 192.4 | 27.3 | 33.6 | 88.4 | -63.6% | 10.0 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 278.1 | 32.7 | 215.2 | 35.8 | 34.3 | 82.4 | -22.6% | 1.84 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 1409 | 101.5 | 235.7 | 26.2 | 74.1 | 480.8 | -83.3% | 15.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q5_p50_us | 3145 | 128.1 | 143.1 | 30.2 | 93.1 | 200.9 | -95.4% | 32.3 | **H2 better** |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 777.7 | 57.7 | 87.6 | 52.1 | 55.0 | 32.5 | -88.7% | 12.6 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5271 | 674.7 | 1726 | 321.0 | 528.3 | 970.7 | -67.3% | 6.71 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q2_p50_us | 634.8 | 45.2 | 147.5 | 43.9 | 44.6 | 129.7 | -76.8% | 10.9 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p50_us | 2097 | 170.0 | 148.6 | 27.4 | 121.8 | 238.6 | -92.9% | 16.0 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 11098 | 2290 | 2175 | 390.0 | 1643 | 3487 | -80.4% | 5.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1732 | 98.3 | 291.6 | 37.1 | 74.3 | 367.3 | -83.2% | 19.4 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 8468 | 3248 | 3160 | 943.7 | 2392 | 2050 | -62.7% | 2.22 | no difference |
| P+ vs H2 | write_p50_us | 946.6 | 62.4 | 1372 | 43.0 | 53.6 | 37.5 | +44.9% | 7.94 | **P+ better** |
| P+ vs H2 | write_p99_us | 2780 | 288.4 | 4902 | 386.5 | 341.0 | 687.7 | +76.3% | 6.22 | **P+ better** |
| P+ vs H2 | commits_per_s | 889.2 | 36.7 | 617.0 | 26.2 | 31.9 | 10.6 | -30.6% | 8.54 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.01 | 43.5 | 0.01 | 0.01 | 0.08 | +0.5% | 24.0 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -2597 | 123.9 | -2672 | 118.2 | 121.1 | 2832 | +2.9% | 0.62 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 111.1 | 10.7 | 701.5 | 39.1 | 28.7 | 46.1 | +531.6% | 20.6 | **N better** |
| N vs P+ | q1_p99_us | 533.0 | 229.6 | 2280 | 610.2 | 461.0 | 1037 | +327.7% | 3.79 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 131.2 | 10.5 | 587.9 | 45.7 | 33.1 | 70.9 | +348.1% | 13.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2227 | 104.6 | 3178 | 118.8 | 111.9 | 710.6 | +42.7% | 8.50 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 136.8 | 17.1 | 911.9 | 71.7 | 52.1 | 125.0 | +566.7% | 14.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 456.2 | 65.1 | 5651 | 1789 | 1266 | 1319 | +1138.7% | 4.10 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 155.3 | 17.0 | 754.9 | 76.8 | 55.6 | 88.5 | +386.0% | 10.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 207.3 | 17.8 | 1881 | 147.8 | 105.3 | 61.2 | +807.1% | 15.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 813.8 | 134.0 | 10454 | 2802 | 1983 | 2310 | +1184.6% | 4.86 | **N better** |
| N vs P+ | c4_q2_p50_us | 228.5 | 27.8 | 1564 | 66.7 | 51.1 | 116.7 | +584.4% | 26.2 | **N better** |
| N vs P+ | c4_q2_p99_us | 1124 | 412.1 | 5504 | 811.7 | 643.7 | 7957 | +389.8% | 6.80 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 438.5 | 14.0 | 912.9 | 50.5 | 37.0 | 133.1 | +108.2% | 12.8 | **N better** |
| N vs P+ | write_p99_us | 1217 | 266.5 | 3295 | 256.9 | 261.7 | 647.8 | +170.7% | 7.94 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1927 | 164.7 | 922.9 | 41.7 | 120.1 | 480.1 | -52.1% | 8.36 | **N better** |
| N vs P+ | pss_mib | 12.9 | 0.29 | 43.3 | 0.01 | 0.20 | 0.22 | +235.5% | 149.6 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | -534.6 | 383.1 | -2592 | 102.9 | 280.5 | 2446 | +384.9% | 7.34 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 111.1 | 10.7 | 110.9 | 16.4 | 13.9 | 33.9 | -0.2% | 0.01 | BELOW FLOOR |
| N vs P | q1_p99_us | 533.0 | 229.6 | 723.0 | 301.1 | 267.8 | 927.6 | +35.6% | 0.71 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 131.2 | 10.5 | 190.7 | 14.8 | 12.8 | 59.7 | +45.3% | 4.64 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2227 | 104.6 | 173.1 | 36.5 | 78.4 | 702.9 | -92.2% | 26.2 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 136.8 | 17.1 | 126.3 | 13.8 | 15.5 | 115.8 | -7.7% | 0.68 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 456.2 | 65.1 | 1601 | 410.8 | 294.1 | 1794 | +250.9% | 3.89 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 155.3 | 17.0 | 245.9 | 15.6 | 16.4 | 76.8 | +58.3% | 5.54 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 207.3 | 17.8 | 127.6 | 28.3 | 23.6 | 64.6 | -38.5% | 3.37 | **P better** |
| N vs P | c4_q1_p99_us | 813.8 | 134.0 | 2127 | 355.6 | 268.7 | 1992 | +161.4% | 4.89 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 228.5 | 27.8 | 239.8 | 42.7 | 36.1 | 95.0 | +4.9% | 0.31 | BELOW FLOOR |
| N vs P | c4_q2_p99_us | 1124 | 412.1 | 1959 | 469.6 | 441.8 | 690.6 | +74.3% | 1.89 | no difference |
| N vs P | write_p50_us | 438.5 | 14.0 | 13109 | 477.2 | 337.6 | 96.1 | +2889.5% | 37.5 | **N better** |
| N vs P | write_p99_us | 1217 | 266.5 | 21643 | 3854 | 2731 | 7107 | +1678.2% | 7.48 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | commits_per_s | 1927 | 164.7 | 74.1 | 3.01 | 116.5 | 480.0 | -96.2% | 15.9 | **N better** |
| N vs P | pss_mib | 12.9 | 0.29 | 43.3 | 0.02 | 0.20 | 0.18 | +235.2% | 149.2 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | -534.6 | 383.1 | -2580 | 78.2 | 276.5 | 2354 | +382.7% | 7.40 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 701.5 | 39.1 | 110.9 | 16.4 | 30.0 | 32.7 | -84.2% | 19.7 | **P better** |
| P+ vs P | q1_p99_us | 2280 | 610.2 | 723.0 | 301.1 | 481.1 | 854.9 | -68.3% | 3.24 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 587.9 | 45.7 | 190.7 | 14.8 | 34.0 | 39.2 | -67.6% | 11.7 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 298.6 | 26.0 | 220.4 | 26.9 | 26.4 | 49.5 | -26.2% | 2.96 | no difference |
| P+ vs P | q4_p50_us | 1418 | 29.5 | 237.3 | 47.4 | 39.5 | 149.5 | -83.3% | 29.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q5_p50_us | 3178 | 118.8 | 173.1 | 36.5 | 87.8 | 108.9 | -94.6% | 34.2 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 911.9 | 71.7 | 126.3 | 13.8 | 51.6 | 58.2 | -86.2% | 15.2 | **P better** |
| P+ vs P | c2_q1_p99_us | 5651 | 1789 | 1601 | 410.8 | 1298 | 2226 | -71.7% | 3.12 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 754.9 | 76.8 | 245.9 | 15.6 | 55.4 | 68.2 | -67.4% | 9.19 | **P better** |
| P+ vs P | c4_q1_p50_us | 1881 | 147.8 | 127.6 | 28.3 | 106.4 | 29.5 | -93.2% | 16.5 | **P better** |
| P+ vs P | c4_q1_p99_us | 10454 | 2802 | 2127 | 355.6 | 1997 | 3035 | -79.7% | 4.17 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1564 | 66.7 | 239.8 | 42.7 | 56.0 | 105.6 | -84.7% | 23.7 | **P better** |
| P+ vs P | c4_q2_p99_us | 5504 | 811.7 | 1959 | 469.6 | 663.1 | 7961 | -64.4% | 5.35 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 912.9 | 50.5 | 13109 | 477.2 | 339.3 | 127.2 | +1336.0% | 35.9 | **P+ better** |
| P+ vs P | write_p99_us | 3295 | 256.9 | 21643 | 3854 | 2731 | 7078 | +556.9% | 6.72 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | commits_per_s | 922.9 | 41.7 | 74.1 | 3.01 | 29.6 | 22.4 | -92.0% | 28.7 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.01 | 43.3 | 0.02 | 0.02 | 0.17 | -0.1% | 2.24 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2592 | 102.9 | -2580 | 78.2 | 91.4 | 2841 | -0.4% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 110.9 | 16.4 | 98.8 | 9.07 | 13.3 | 6.86 | -10.9% | 0.91 | no difference |
| P vs M | q1_p99_us | 723.0 | 301.1 | 537.4 | 105.1 | 225.5 | 836.8 | -25.7% | 0.82 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 190.7 | 14.8 | 182.9 | 8.95 | 12.2 | 20.0 | -4.1% | 0.64 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 220.4 | 26.9 | 198.8 | 21.7 | 24.5 | 99.3 | -9.8% | 0.89 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 237.3 | 47.4 | 211.5 | 25.1 | 37.9 | 97.6 | -10.9% | 0.68 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q5_p50_us | 173.1 | 36.5 | 126.0 | 11.0 | 26.9 | 47.8 | -27.2% | 1.75 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 126.3 | 13.8 | 75.3 | 16.5 | 15.2 | 70.7 | -40.4% | 3.35 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1601 | 410.8 | 2169 | 678.7 | 561.0 | 2320 | +35.5% | 1.01 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 245.9 | 15.6 | 183.5 | 42.8 | 32.2 | 90.9 | -25.4% | 1.94 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 127.6 | 28.3 | 165.0 | 30.4 | 29.4 | 34.9 | +29.4% | 1.28 | no difference |
| P vs M | c4_q1_p99_us | 2127 | 355.6 | 2270 | 287.2 | 323.2 | 2137 | +6.7% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p50_us | 239.8 | 42.7 | 304.7 | 39.3 | 41.1 | 58.9 | +27.1% | 1.58 | no difference |
| P vs M | c4_q2_p99_us | 1959 | 469.6 | 2658 | 473.0 | 471.3 | 925.6 | +35.7% | 1.48 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 13109 | 477.2 | 13037 | 451.1 | 464.4 | 505.5 | -0.5% | 0.15 | BELOW FLOOR |
| P vs M | write_p99_us | 21643 | 3854 | 18259 | 964.1 | 2809 | 7610 | -15.6% | 1.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | commits_per_s | 74.1 | 3.01 | 74.9 | 1.83 | 2.49 | 14.4 | +1.1% | 0.33 | BELOW FLOOR |
| P vs M | pss_mib | 43.3 | 0.02 | 43.3 | 0.01 | 0.02 | 0.10 | +0.1% | 3.78 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2580 | 78.2 | -2573 | 121.8 | 102.4 | 2728 | -0.3% | 0.07 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 111.1 | 10.7 | 292.5 | 30.0 | 22.5 | 59.1 | +163.4% | 8.05 | **N better** |
| N vs H3 | q1_p99_us | 533.0 | 229.6 | 814.3 | 145.5 | 192.2 | 802.7 | +52.8% | 1.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 131.2 | 10.5 | 333.6 | 32.8 | 24.4 | 79.2 | +154.3% | 8.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2227 | 104.6 | 3380 | 133.8 | 120.1 | 703.0 | +51.8% | 9.60 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 136.8 | 17.1 | 283.2 | 71.0 | 51.6 | 113.4 | +107.1% | 2.84 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 456.2 | 65.1 | 947.8 | 115.5 | 93.8 | 108.6 | +107.8% | 5.24 | **N better** |
| N vs H3 | c2_q2_p50_us | 155.3 | 17.0 | 272.9 | 61.4 | 45.1 | 72.0 | +75.7% | 2.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 207.3 | 17.8 | 483.2 | 63.4 | 46.6 | 59.5 | +133.1% | 5.93 | **N better** |
| N vs H3 | c4_q1_p99_us | 813.8 | 134.0 | 1962 | 363.2 | 273.7 | 490.6 | +141.1% | 4.19 | **N better** |
| N vs H3 | c4_q2_p50_us | 228.5 | 27.8 | 514.5 | 77.2 | 58.0 | 91.3 | +125.1% | 4.93 | **N better** |
| N vs H3 | c4_q2_p99_us | 1124 | 412.1 | 2003 | 427.6 | 419.9 | 554.5 | +78.2% | 2.09 | no difference |
| N vs H3 | write_p50_us | 438.5 | 14.0 | 1276 | 69.3 | 50.0 | 151.7 | +191.1% | 16.8 | **N better** |
| N vs H3 | write_p99_us | 1217 | 266.5 | 4649 | 1339 | 965.1 | 737.1 | +281.9% | 3.56 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1927 | 164.7 | 629.3 | 86.8 | 131.6 | 480.1 | -67.3% | 9.86 | **N better** |
| N vs H3 | pss_mib | 12.9 | 0.29 | 67.1 | 0.21 | 0.25 | 0.18 | +419.7% | 214.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | -534.6 | 383.1 | 16753 | 1163 | 865.5 | 10163 | -3234.0% | 20.0 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 701.5 | 39.1 | 292.5 | 30.0 | 34.9 | 58.4 | -58.3% | 11.7 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2280 | 610.2 | 814.3 | 145.5 | 443.5 | 717.4 | -64.3% | 3.30 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 587.9 | 45.7 | 333.6 | 32.8 | 39.8 | 65.1 | -43.3% | 6.39 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 298.6 | 26.0 | 353.6 | 55.9 | 43.6 | 81.5 | +18.4% | 1.26 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1418 | 29.5 | 1370 | 84.5 | 63.3 | 235.1 | -3.4% | 0.76 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3178 | 118.8 | 3380 | 133.8 | 126.5 | 109.3 | +6.4% | 1.60 | no difference |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 911.9 | 71.7 | 283.2 | 71.0 | 71.3 | 53.1 | -68.9% | 8.81 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5651 | 1789 | 947.8 | 115.5 | 1268 | 1323 | -83.2% | 3.71 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p50_us | 754.9 | 76.8 | 272.9 | 61.4 | 69.5 | 62.8 | -63.8% | 6.93 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1881 | 147.8 | 483.2 | 63.4 | 113.7 | 15.5 | -74.3% | 12.3 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 10454 | 2802 | 1962 | 363.2 | 1998 | 2342 | -81.2% | 4.25 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1564 | 66.7 | 514.5 | 77.2 | 72.1 | 102.4 | -67.1% | 14.5 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5504 | 811.7 | 2003 | 427.6 | 648.7 | 7951 | -63.6% | 5.40 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 912.9 | 50.5 | 1276 | 69.3 | 60.6 | 173.2 | +39.8% | 6.00 | **P+ better** |
| P+ vs H3 | write_p99_us | 3295 | 256.9 | 4649 | 1339 | 963.7 | 352.0 | +41.1% | 1.40 | no difference |
| P+ vs H3 | commits_per_s | 922.9 | 41.7 | 629.3 | 86.8 | 68.1 | 24.1 | -31.8% | 4.31 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.01 | 67.1 | 0.21 | 0.15 | 0.17 | +54.9% | 157.8 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2592 | 102.9 | 16753 | 1163 | 825.2 | 10287 | -746.3% | 23.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 111.1 | 10.7 | 1283 | 103.9 | 73.9 | 175.3 | +1054.8% | 15.9 | **N better** |
| N vs T | q1_p99_us | 533.0 | 229.6 | 3264 | 797.0 | 586.5 | 1729 | +512.3% | 4.66 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | q2_p50_us | 131.2 | 10.5 | 1301 | 88.5 | 63.0 | 218.7 | +891.3% | 18.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 136.8 | 17.1 | 1936 | 77.7 | 56.3 | 113.9 | +1315.1% | 32.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q1_p99_us | 456.2 | 65.1 | 5775 | 1895 | 1341 | 843.7 | +1166.0% | 3.97 | **N better** |
| N vs T | c2_q2_p50_us | 155.3 | 17.0 | 1988 | 135.2 | 96.4 | 82.2 | +1180.0% | 19.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p50_us | 207.3 | 17.8 | 2278 | 129.1 | 92.2 | 190.7 | +999.0% | 22.5 | **N better** |
| N vs T | c4_q1_p99_us | 813.8 | 134.0 | 4844 | 1171 | 833.5 | 2466 | +495.3% | 4.84 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c4_q2_p50_us | 228.5 | 27.8 | 2362 | 114.8 | 83.6 | 162.7 | +933.5% | 25.5 | **N better** |
| N vs T | c4_q2_p99_us | 1124 | 412.1 | 5263 | 1219 | 909.7 | 3186 | +368.4% | 4.55 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 438.5 | 14.0 | 1221 | 86.0 | 61.6 | 73.7 | +178.4% | 12.7 | **N better** |
| N vs T | write_p99_us | 1217 | 266.5 | 2169 | 318.2 | 293.5 | 1733 | +78.2% | 3.24 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | commits_per_s | 1927 | 164.7 | 789.4 | 69.2 | 126.3 | 499.8 | -59.0% | 9.00 | **N better** |
| N vs T | pss_mib | 12.9 | 0.29 | 2529 | 0.04 | 0.21 | 1.85 | +19493.6% | 12244 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | -534.6 | 383.1 | -3610 | 180.0 | 299.3 | 1990 | +575.4% | 10.3 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 701.5 | 39.1 | 1283 | 103.9 | 78.5 | 175.0 | +82.8% | 7.40 | **P+ better** |
| P+ vs T | q1_p99_us | 2280 | 610.2 | 3264 | 797.0 | 709.8 | 1691 | +43.1% | 1.39 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | q2_p50_us | 587.9 | 45.7 | 1301 | 88.5 | 70.4 | 214.0 | +121.2% | 10.1 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 911.9 | 71.7 | 1936 | 77.7 | 74.8 | 54.2 | +112.3% | 13.7 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 5651 | 1789 | 5775 | 1895 | 1843 | 1565 | +2.2% | 0.07 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c2_q2_p50_us | 754.9 | 76.8 | 1988 | 135.2 | 109.9 | 74.3 | +163.4% | 11.2 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 1881 | 147.8 | 2278 | 129.1 | 138.8 | 181.9 | +21.2% | 2.87 | no difference |
| P+ vs T | c4_q1_p99_us | 10454 | 2802 | 4844 | 1171 | 2147 | 3365 | -53.7% | 2.61 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c4_q2_p50_us | 1564 | 66.7 | 2362 | 114.8 | 93.9 | 169.2 | +51.0% | 8.50 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 5504 | 811.7 | 5263 | 1219 | 1035 | 8547 | -4.4% | 0.23 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | write_p50_us | 912.9 | 50.5 | 1221 | 86.0 | 70.5 | 111.3 | +33.7% | 4.37 | **P+ better** |
| P+ vs T | write_p99_us | 3295 | 256.9 | 2169 | 318.2 | 289.2 | 1607 | -34.2% | 3.89 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 922.9 | 41.7 | 789.4 | 69.2 | 57.1 | 141.0 | -14.5% | 2.34 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.01 | 2529 | 0.04 | 0.03 | 1.85 | +5740.5% | 76630 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -2592 | 102.9 | -3610 | 180.0 | 146.6 | 2547 | +39.3% | 6.95 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 292.5 | 30.0 | 1283 | 103.9 | 76.5 | 178.9 | +338.5% | 12.9 | **H3 better** |
| H3 vs T | q1_p99_us | 814.3 | 145.5 | 3264 | 797.0 | 572.9 | 1558 | +300.8% | 4.28 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | q2_p50_us | 333.6 | 32.8 | 1301 | 88.5 | 66.8 | 216.9 | +289.9% | 14.5 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 283.2 | 71.0 | 1936 | 77.7 | 74.5 | 12.8 | +583.4% | 22.2 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 947.8 | 115.5 | 5775 | 1895 | 1342 | 850.6 | +509.4% | 3.60 | **H3 better** |
| H3 vs T | c2_q2_p50_us | 272.9 | 61.4 | 1988 | 135.2 | 105.0 | 53.5 | +628.6% | 16.3 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 483.2 | 63.4 | 2278 | 129.1 | 101.7 | 181.3 | +371.5% | 17.6 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 1962 | 363.2 | 4844 | 1171 | 867.0 | 2496 | +147.0% | 3.32 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c4_q2_p50_us | 514.5 | 77.2 | 2362 | 114.8 | 97.9 | 152.8 | +359.1% | 18.9 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 2003 | 427.6 | 5263 | 1219 | 913.3 | 3170 | +162.8% | 3.57 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | write_p50_us | 1276 | 69.3 | 1221 | 86.0 | 78.1 | 133.0 | -4.4% | 0.71 | BELOW FLOOR |
| H3 vs T | write_p99_us | 4649 | 1339 | 2169 | 318.2 | 972.9 | 1645 | -53.3% | 2.55 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 629.3 | 86.8 | 789.4 | 69.2 | 78.5 | 140.9 | +25.4% | 2.04 | no difference |
| H3 vs T | pss_mib | 67.1 | 0.21 | 2529 | 0.04 | 0.15 | 1.85 | +3670.4% | 16001 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 16753 | 1163 | -3610 | 180.0 | 831.8 | 10188 | -121.6% | 24.5 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 111.1 | 10.7 | 800.7 | 44.6 | 32.5 | 42.6 | +620.9% | 21.2 | **N better** |
| N vs H1 | q1_p99_us | 533.0 | 229.6 | 2638 | 824.1 | 604.9 | 780.5 | +395.0% | 3.48 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p50_us | 131.2 | 10.5 | 697.7 | 60.0 | 43.1 | 102.0 | +431.9% | 13.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2227 | 104.6 | 500.9 | 66.3 | 87.6 | 707.1 | -77.5% | 19.7 | **H1 better** |
| N vs H1 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | c2_q1_p50_us | 136.8 | 17.1 | 917.6 | 49.5 | 37.1 | 182.6 | +570.9% | 21.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q1_p99_us | 456.2 | 65.1 | 2650 | 447.2 | 319.6 | 2340 | +481.0% | 6.87 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c2_q2_p50_us | 155.3 | 17.0 | 866.0 | 81.6 | 59.0 | 215.2 | +457.5% | 12.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p50_us | 207.3 | 17.8 | 902.8 | 79.1 | 57.3 | 159.7 | +335.5% | 12.1 | **N better** |
| N vs H1 | c4_q1_p99_us | 813.8 | 134.0 | 2625 | 542.7 | 395.3 | 1437 | +222.6% | 4.58 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c4_q2_p50_us | 228.5 | 27.8 | 880.0 | 26.3 | 27.1 | 101.9 | +285.1% | 24.1 | **N better** |
| N vs H1 | c4_q2_p99_us | 1124 | 412.1 | 3712 | 805.4 | 639.7 | 1013 | +230.3% | 4.05 | **N better** |
| N vs H1 | write_p50_us | 438.5 | 14.0 | 1364 | 85.5 | 61.3 | 108.0 | +211.0% | 15.1 | **N better** |
| N vs H1 | write_p99_us | 1217 | 266.5 | 4277 | 467.1 | 380.3 | 927.4 | +251.4% | 8.05 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | commits_per_s | 1927 | 164.7 | 629.2 | 45.7 | 120.8 | 479.8 | -67.3% | 10.7 | **N better** |
| N vs H1 | pss_mib | 12.9 | 0.29 | 288.6 | 9.79 | 6.93 | 6.62 | +2135.9% | 39.8 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | -534.6 | 383.1 | 41257 | 7425 | 5257 | 6787 | -7818.0% | 7.95 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H1 | q1_p50_us | 701.5 | 39.1 | 800.7 | 44.6 | 42.0 | 41.7 | +14.1% | 2.36 | no difference |
| P+ vs H1 | q1_p99_us | 2280 | 610.2 | 2638 | 824.1 | 725.1 | 692.4 | +15.7% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q2_p50_us | 587.9 | 45.7 | 697.7 | 60.0 | 53.4 | 91.5 | +18.7% | 2.06 | no difference |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 298.6 | 26.0 | 746.3 | 65.4 | 49.8 | 73.0 | +149.9% | 9.00 | **P+ better** |
| P+ vs H1 | q4_p50_us | 1418 | 29.5 | 3131 | 349.9 | 248.3 | 718.0 | +120.8% | 6.90 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3178 | 118.8 | 500.9 | 66.3 | 96.2 | 133.3 | -84.2% | 27.8 | **H1 better** |
| P+ vs H1 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | c2_q1_p50_us | 911.9 | 71.7 | 917.6 | 49.5 | 61.6 | 152.7 | +0.6% | 0.09 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 5651 | 1789 | 2650 | 447.2 | 1304 | 2686 | -53.1% | 2.30 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c2_q2_p50_us | 754.9 | 76.8 | 866.0 | 81.6 | 79.3 | 212.3 | +14.7% | 1.40 | BELOW FLOOR |
| P+ vs H1 | c4_q1_p50_us | 1881 | 147.8 | 902.8 | 79.1 | 118.5 | 149.0 | -52.0% | 8.25 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 10454 | 2802 | 2625 | 542.7 | 2018 | 2703 | -74.9% | 3.88 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | c4_q2_p50_us | 1564 | 66.7 | 880.0 | 26.3 | 50.7 | 111.9 | -43.7% | 13.5 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 5504 | 811.7 | 3712 | 805.4 | 808.6 | 7996 | -32.6% | 2.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | write_p50_us | 912.9 | 50.5 | 1364 | 85.5 | 70.2 | 136.5 | +49.4% | 6.42 | **P+ better** |
| P+ vs H1 | write_p99_us | 3295 | 256.9 | 4277 | 467.1 | 376.9 | 663.9 | +29.8% | 2.61 | no difference |
| P+ vs H1 | commits_per_s | 922.9 | 41.7 | 629.2 | 45.7 | 43.8 | 17.2 | -31.8% | 6.71 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.01 | 288.6 | 9.79 | 6.92 | 6.62 | +566.5% | 35.4 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -2592 | 102.9 | 41257 | 7425 | 5251 | 6971 | -1691.7% | 8.35 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 111.1 | 10.7 | 105.7 | 17.6 | 14.6 | 33.2 | -4.8% | 0.37 | BELOW FLOOR |
| N vs H2 | q1_p99_us | 533.0 | 229.6 | 702.0 | 295.8 | 264.8 | 778.9 | +31.7% | 0.64 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 131.2 | 10.5 | 187.4 | 24.2 | 18.6 | 65.4 | +42.9% | 3.02 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2227 | 104.6 | 142.1 | 23.8 | 75.9 | 702.6 | -93.6% | 27.5 | **H2 better** |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 136.8 | 17.1 | 110.5 | 21.9 | 19.6 | 113.2 | -19.2% | 1.34 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q1_p99_us | 456.2 | 65.1 | 1976 | 344.9 | 248.2 | 649.8 | +333.2% | 6.12 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q2_p50_us | 155.3 | 17.0 | 205.4 | 16.7 | 16.9 | 67.8 | +32.2% | 2.96 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p50_us | 207.3 | 17.8 | 139.1 | 31.3 | 25.4 | 122.7 | -32.9% | 2.68 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p99_us | 813.8 | 134.0 | 2409 | 388.8 | 290.8 | 1619 | +196.0% | 5.49 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p50_us | 228.5 | 27.8 | 261.6 | 20.0 | 24.2 | 81.6 | +14.4% | 1.36 | BELOW FLOOR |
| N vs H2 | c4_q2_p99_us | 1124 | 412.1 | 2460 | 349.0 | 381.9 | 572.5 | +118.9% | 3.50 | **N better** |
| N vs H2 | write_p50_us | 438.5 | 14.0 | 1491 | 49.2 | 36.2 | 98.8 | +240.1% | 29.1 | **N better** |
| N vs H2 | write_p99_us | 1217 | 266.5 | 4950 | 373.3 | 324.3 | 1304 | +306.7% | 11.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1927 | 164.7 | 572.0 | 23.8 | 117.6 | 493.5 | -70.3% | 11.5 | **N better** |
| N vs H2 | pss_mib | 12.9 | 0.29 | 43.4 | 0.03 | 0.20 | 0.16 | +236.5% | 149.7 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | -534.6 | 383.1 | -2620 | 138.1 | 287.9 | 2329 | +390.2% | 7.24 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 701.5 | 39.1 | 105.7 | 17.6 | 30.3 | 32.0 | -84.9% | 19.6 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2280 | 610.2 | 702.0 | 295.8 | 479.5 | 690.6 | -69.2% | 3.29 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 587.9 | 45.7 | 187.4 | 24.2 | 36.5 | 47.3 | -68.1% | 11.0 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 298.6 | 26.0 | 217.9 | 15.3 | 21.3 | 47.5 | -27.0% | 3.79 | **H2 better** |
| P+ vs H2 | q4_p50_us | 1418 | 29.5 | 259.7 | 44.0 | 37.4 | 118.9 | -81.7% | 30.9 | **H2 better** |
| P+ vs H2 | q5_p50_us | 3178 | 118.8 | 142.1 | 23.8 | 85.6 | 106.4 | -95.5% | 35.4 | **H2 better** |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 911.9 | 71.7 | 110.5 | 21.9 | 53.0 | 52.9 | -87.9% | 15.1 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5651 | 1789 | 1976 | 344.9 | 1289 | 1470 | -65.0% | 2.85 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 754.9 | 76.8 | 205.4 | 16.7 | 55.6 | 57.9 | -72.8% | 9.89 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1881 | 147.8 | 139.1 | 31.3 | 106.8 | 108.4 | -92.6% | 16.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 10454 | 2802 | 2409 | 388.8 | 2000 | 2804 | -77.0% | 4.02 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p50_us | 1564 | 66.7 | 261.6 | 20.0 | 49.2 | 93.8 | -83.3% | 26.5 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 5504 | 811.7 | 2460 | 349.0 | 624.8 | 7952 | -55.3% | 4.87 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 912.9 | 50.5 | 1491 | 49.2 | 49.8 | 129.3 | +63.4% | 11.6 | **P+ better** |
| P+ vs H2 | write_p99_us | 3295 | 256.9 | 4950 | 373.3 | 320.4 | 1132 | +50.2% | 5.17 | **P+ better** |
| P+ vs H2 | commits_per_s | 922.9 | 41.7 | 572.0 | 23.8 | 34.0 | 116.8 | -38.0% | 10.3 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.01 | 43.4 | 0.03 | 0.02 | 0.15 | +0.3% | 6.73 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | -2592 | 102.9 | -2620 | 138.1 | 121.8 | 2820 | +1.1% | 0.23 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 111.0 | 13.1 | 705.5 | 41.3 | 30.6 | 182.2 | +535.4% | 19.4 | **N better** |
| N vs P+ | q1_p99_us | 360.4 | 68.3 | 2020 | 624.3 | 444.1 | 952.5 | +460.6% | 3.74 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 124.1 | 13.0 | 535.3 | 29.0 | 22.4 | 51.5 | +331.2% | 18.3 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2125 | 83.3 | 3210 | 156.8 | 125.5 | 661.8 | +51.1% | 8.65 | **N better** |
| N vs P+ | q6_p50_us | 2821 | 206.3 | 3194 | 199.7 | 203.0 | 833.4 | +13.2% | 1.84 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 121.2 | 27.9 | 894.6 | 110.3 | 80.5 | 19.0 | +638.4% | 9.61 | **N better** |
| N vs P+ | c2_q1_p99_us | 614.4 | 119.5 | 5821 | 798.6 | 571.0 | 1025 | +847.3% | 9.12 | **N better** |
| N vs P+ | c2_q2_p50_us | 134.9 | 45.4 | 717.1 | 74.5 | 61.7 | 258.6 | +431.5% | 9.44 | **N better** |
| N vs P+ | c4_q1_p50_us | 220.2 | 60.0 | 1974 | 62.7 | 61.4 | 141.8 | +796.2% | 28.6 | **N better** |
| N vs P+ | c4_q1_p99_us | 898.9 | 105.8 | 9849 | 3141 | 2222 | 21784 | +995.8% | 4.03 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 237.5 | 62.2 | 1652 | 67.9 | 65.1 | 94.3 | +595.8% | 21.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 1117 | 368.5 | 8449 | 1830 | 1320 | 542.9 | +656.6% | 5.56 | **N better** |
| N vs P+ | write_p50_us | 418.2 | 37.7 | 945.5 | 36.7 | 37.2 | 152.3 | +126.1% | 14.2 | **N better** |
| N vs P+ | write_p99_us | 1174 | 437.4 | 3262 | 249.6 | 356.1 | 640.8 | +177.8% | 5.86 | **N better** |
| N vs P+ | commits_per_s | 1924 | 388.8 | 916.8 | 10.3 | 275.0 | 402.9 | -52.3% | 3.66 | **N better** |
| N vs P+ | pss_mib | 13.5 | 0.26 | 43.3 | 0.02 | 0.19 | 1.12 | +220.7% | 159.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 9120 | 484.8 | -2692 | 85.9 | 348.1 | 3764 | -129.5% | 33.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 111.0 | 13.1 | 106.3 | 19.2 | 16.4 | 32.7 | -4.3% | 0.29 | BELOW FLOOR |
| N vs P | q1_p99_us | 360.4 | 68.3 | 902.4 | 187.4 | 141.0 | 876.7 | +150.4% | 3.84 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 124.1 | 13.0 | 187.7 | 20.5 | 17.2 | 22.4 | +51.2% | 3.70 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2125 | 83.3 | 142.2 | 21.4 | 60.8 | 163.3 | -93.3% | 32.6 | **P better** |
| N vs P | q6_p50_us | 2821 | 206.3 | 554.5 | 66.4 | 153.2 | 555.3 | -80.3% | 14.8 | **P better** |
| N vs P | c2_q1_p50_us | 121.2 | 27.9 | 99.4 | 45.2 | 37.5 | 5.07 | -18.0% | 0.58 | no difference |
| N vs P | c2_q1_p99_us | 614.4 | 119.5 | 1515 | 387.0 | 286.4 | 1464 | +146.6% | 3.14 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 134.9 | 45.4 | 183.3 | 61.0 | 53.8 | 61.2 | +35.8% | 0.90 | BELOW FLOOR |
| N vs P | c4_q1_p50_us | 220.2 | 60.0 | 144.8 | 17.8 | 44.2 | 59.4 | -34.3% | 1.70 | no difference |
| N vs P | c4_q1_p99_us | 898.9 | 105.8 | 2266 | 234.0 | 181.6 | 1547 | +152.1% | 7.53 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 237.5 | 62.2 | 235.5 | 39.6 | 52.1 | 59.5 | -0.8% | 0.04 | BELOW FLOOR |
| N vs P | c4_q2_p99_us | 1117 | 368.5 | 3288 | 997.7 | 752.1 | 1111 | +194.4% | 2.89 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 418.2 | 37.7 | 13554 | 757.8 | 536.5 | 1901 | +3141.0% | 24.5 | **N better** |
| N vs P | write_p99_us | 1174 | 437.4 | 20917 | 3075 | 2197 | 6054 | +1681.4% | 8.99 | **N better** |
| N vs P | commits_per_s | 1924 | 388.8 | 71.7 | 2.13 | 274.9 | 353.3 | -96.3% | 6.74 | **N better** |
| N vs P | pss_mib | 13.5 | 0.26 | 43.4 | 0.04 | 0.19 | 1.12 | +221.4% | 158.2 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 9120 | 484.8 | -2398 | 96.9 | 349.6 | 3719 | -126.3% | 32.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 705.5 | 41.3 | 106.3 | 19.2 | 32.2 | 185.1 | -84.9% | 18.6 | **P better** |
| P+ vs P | q1_p99_us | 2020 | 624.3 | 902.4 | 187.4 | 460.9 | 1031 | -55.3% | 2.43 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 535.3 | 29.0 | 187.7 | 20.5 | 25.1 | 51.0 | -64.9% | 13.8 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 271.9 | 47.6 | 216.5 | 34.6 | 41.6 | 69.8 | -20.4% | 1.33 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1411 | 76.5 | 169.1 | 26.8 | 57.3 | 190.8 | -88.0% | 21.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q5_p50_us | 3210 | 156.8 | 142.2 | 21.4 | 111.9 | 642.4 | -95.6% | 27.4 | **P better** |
| P+ vs P | q6_p50_us | 3194 | 199.7 | 554.5 | 66.4 | 148.8 | 621.5 | -82.6% | 17.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 894.6 | 110.3 | 99.4 | 45.2 | 84.3 | 18.3 | -88.9% | 9.43 | **P better** |
| P+ vs P | c2_q1_p99_us | 5821 | 798.6 | 1515 | 387.0 | 627.5 | 1784 | -74.0% | 6.86 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 717.1 | 74.5 | 183.3 | 61.0 | 68.1 | 257.1 | -74.4% | 7.84 | **P better** |
| P+ vs P | c4_q1_p50_us | 1974 | 62.7 | 144.8 | 17.8 | 46.1 | 134.5 | -92.7% | 39.7 | **P better** |
| P+ vs P | c4_q1_p99_us | 9849 | 3141 | 2266 | 234.0 | 2227 | 21736 | -77.0% | 3.41 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1652 | 67.9 | 235.5 | 39.6 | 55.6 | 111.3 | -85.7% | 25.5 | **P better** |
| P+ vs P | c4_q2_p99_us | 8449 | 1830 | 3288 | 997.7 | 1474 | 977.8 | -61.1% | 3.50 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 945.5 | 36.7 | 13554 | 757.8 | 536.5 | 1902 | +1333.5% | 23.5 | **P+ better** |
| P+ vs P | write_p99_us | 3262 | 249.6 | 20917 | 3075 | 2182 | 6081 | +541.2% | 8.09 | **P+ better** |
| P+ vs P | commits_per_s | 916.8 | 10.3 | 71.7 | 2.13 | 7.42 | 193.8 | -92.2% | 113.8 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.02 | 43.4 | 0.04 | 0.03 | 0.04 | +0.2% | 2.95 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2692 | 85.9 | -2398 | 96.9 | 91.6 | 2420 | -10.9% | 3.20 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 106.3 | 19.2 | 102.8 | 14.0 | 16.8 | 38.3 | -3.2% | 0.21 | BELOW FLOOR |
| P vs M | q1_p99_us | 902.4 | 187.4 | 834.9 | 199.3 | 193.4 | 725.1 | -7.5% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 187.7 | 20.5 | 205.8 | 21.1 | 20.8 | 16.0 | +9.7% | 0.87 | no difference |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 216.5 | 34.6 | 218.1 | 20.2 | 28.3 | 38.4 | +0.7% | 0.06 | BELOW FLOOR |
| P vs M | q4_p50_us | 169.1 | 26.8 | 205.7 | 32.7 | 29.9 | 138.4 | +21.7% | 1.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q5_p50_us | 142.2 | 21.4 | 133.6 | 12.0 | 17.4 | 54.0 | -6.1% | 0.50 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | 554.5 | 66.4 | 474.7 | 68.2 | 67.3 | 247.9 | -14.4% | 1.19 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p50_us | 99.4 | 45.2 | 103.6 | 29.8 | 38.3 | 8.76 | +4.3% | 0.11 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1515 | 387.0 | 2011 | 539.4 | 469.4 | 1462 | +32.7% | 1.06 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 183.3 | 61.0 | 185.0 | 26.3 | 47.0 | 39.4 | +0.9% | 0.04 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 144.8 | 17.8 | 103.1 | 27.8 | 23.3 | 27.6 | -28.8% | 1.79 | no difference |
| P vs M | c4_q1_p99_us | 2266 | 234.0 | 1850 | 247.4 | 240.8 | 1248 | -18.3% | 1.73 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 235.5 | 39.6 | 217.3 | 44.3 | 42.0 | 66.1 | -7.7% | 0.43 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3288 | 997.7 | 2525 | 439.3 | 770.8 | 994.0 | -23.2% | 0.99 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 13554 | 757.8 | 12912 | 379.5 | 599.3 | 1903 | -4.7% | 1.07 | BELOW FLOOR |
| P vs M | write_p99_us | 20917 | 3075 | 17081 | 1266 | 2352 | 9250 | -18.3% | 1.63 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | commits_per_s | 71.7 | 2.13 | 74.2 | 4.60 | 3.59 | 3.44 | +3.4% | 0.69 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.04 | 43.5 | 0.01 | 0.03 | 0.05 | +0.3% | 3.83 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2398 | 96.9 | -2337 | 111.4 | 104.4 | 2222 | -2.6% | 0.59 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 111.0 | 13.1 | 303.9 | 30.3 | 23.3 | 64.8 | +173.7% | 8.27 | **N better** |
| N vs H3 | q1_p99_us | 360.4 | 68.3 | 1258 | 319.9 | 231.3 | 690.1 | +249.1% | 3.88 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 124.1 | 13.0 | 321.9 | 34.0 | 25.8 | 30.4 | +159.3% | 7.68 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2125 | 83.3 | 3435 | 395.2 | 285.6 | 893.5 | +61.7% | 4.59 | **N better** |
| N vs H3 | q6_p50_us | 2821 | 206.3 | 3908 | 504.0 | 385.1 | 555.7 | +38.5% | 2.82 | no difference |
| N vs H3 | c2_q1_p50_us | 121.2 | 27.9 | 327.2 | 38.2 | 33.5 | 66.9 | +170.1% | 6.16 | **N better** |
| N vs H3 | c2_q1_p99_us | 614.4 | 119.5 | 1175 | 320.0 | 241.5 | 85.9 | +91.2% | 2.32 | no difference |
| N vs H3 | c2_q2_p50_us | 134.9 | 45.4 | 351.5 | 25.5 | 36.8 | 199.0 | +160.5% | 5.88 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 220.2 | 60.0 | 450.8 | 20.4 | 44.8 | 259.4 | +104.7% | 5.15 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 898.9 | 105.8 | 2084 | 359.0 | 264.6 | 1777 | +131.8% | 4.48 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 237.5 | 62.2 | 476.7 | 39.8 | 52.2 | 105.1 | +100.7% | 4.58 | **N better** |
| N vs H3 | c4_q2_p99_us | 1117 | 368.5 | 1790 | 298.0 | 335.1 | 695.5 | +60.3% | 2.01 | BELOW FLOOR |
| N vs H3 | write_p50_us | 418.2 | 37.7 | 1229 | 46.2 | 42.1 | 108.8 | +193.9% | 19.2 | **N better** |
| N vs H3 | write_p99_us | 1174 | 437.4 | 6236 | 2724 | 1951 | 306.2 | +431.1% | 2.59 | no difference |
| N vs H3 | commits_per_s | 1924 | 388.8 | 587.7 | 100.4 | 284.0 | 355.0 | -69.4% | 4.70 | **N better** |
| N vs H3 | pss_mib | 13.5 | 0.26 | 67.6 | 0.09 | 0.20 | 1.12 | +401.1% | 273.5 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 9120 | 484.8 | 16039 | 322.0 | 411.5 | 10563 | +75.9% | 16.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q1_p50_us | 705.5 | 41.3 | 303.9 | 30.3 | 36.2 | 193.3 | -56.9% | 11.1 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2020 | 624.3 | 1258 | 319.9 | 496.0 | 877.6 | -37.7% | 1.54 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 535.3 | 29.0 | 321.9 | 34.0 | 31.6 | 55.0 | -39.9% | 6.75 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 271.9 | 47.6 | 327.7 | 35.6 | 42.0 | 66.8 | +20.5% | 1.33 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1411 | 76.5 | 1393 | 82.5 | 79.5 | 131.6 | -1.3% | 0.22 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3210 | 156.8 | 3435 | 395.2 | 300.6 | 1088 | +7.0% | 0.75 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 3194 | 199.7 | 3908 | 504.0 | 383.3 | 621.8 | +22.3% | 1.86 | no difference |
| P+ vs H3 | c2_q1_p50_us | 894.6 | 110.3 | 327.2 | 38.2 | 82.6 | 69.2 | -63.4% | 6.87 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5821 | 798.6 | 1175 | 320.0 | 608.3 | 1023 | -79.8% | 7.64 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 717.1 | 74.5 | 351.5 | 25.5 | 55.7 | 319.4 | -51.0% | 6.57 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1974 | 62.7 | 450.8 | 20.4 | 46.6 | 286.1 | -77.2% | 32.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 9849 | 3141 | 2084 | 359.0 | 2235 | 21753 | -78.8% | 3.47 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1652 | 67.9 | 476.7 | 39.8 | 55.7 | 141.0 | -71.2% | 21.1 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8449 | 1830 | 1790 | 298.0 | 1311 | 453.9 | -78.8% | 5.08 | **H3 better** |
| P+ vs H3 | write_p50_us | 945.5 | 36.7 | 1229 | 46.2 | 41.7 | 122.9 | +30.0% | 6.80 | **P+ better** |
| P+ vs H3 | write_p99_us | 3262 | 249.6 | 6236 | 2724 | 1934 | 642.4 | +91.1% | 1.54 | no difference |
| P+ vs H3 | commits_per_s | 916.8 | 10.3 | 587.7 | 100.4 | 71.4 | 196.9 | -35.9% | 4.61 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.02 | 67.6 | 0.09 | 0.07 | 0.07 | +56.2% | 357.4 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2692 | 85.9 | 16039 | 322.0 | 235.7 | 10179 | -695.9% | 79.5 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 111.0 | 13.1 | 1349 | 107.2 | 76.3 | 46.8 | +1115.2% | 16.2 | **N better** |
| N vs T | q1_p99_us | 360.4 | 68.3 | 3022 | 554.3 | 394.9 | 714.2 | +738.5% | 6.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q2_p50_us | 124.1 | 13.0 | 1391 | 95.2 | 67.9 | 243.6 | +1020.8% | 18.7 | **N better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 121.2 | 27.9 | 1907 | 107.5 | 78.5 | 228.4 | +1473.7% | 22.7 | **N better** |
| N vs T | c2_q1_p99_us | 614.4 | 119.5 | 5282 | 2042 | 1447 | 1358 | +759.7% | 3.23 | **N better** |
| N vs T | c2_q2_p50_us | 134.9 | 45.4 | 1888 | 80.3 | 65.2 | 504.2 | +1299.1% | 26.9 | **N better** |
| N vs T | c4_q1_p50_us | 220.2 | 60.0 | 2242 | 147.7 | 112.7 | 197.3 | +917.9% | 17.9 | **N better** |
| N vs T | c4_q1_p99_us | 898.9 | 105.8 | 5092 | 1075 | 763.5 | 1530 | +466.5% | 5.49 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p50_us | 237.5 | 62.2 | 2252 | 107.9 | 88.1 | 426.4 | +848.4% | 22.9 | **N better** |
| N vs T | c4_q2_p99_us | 1117 | 368.5 | 5526 | 861.1 | 662.3 | 1533 | +394.8% | 6.66 | **N better** |
| N vs T | write_p50_us | 418.2 | 37.7 | 1270 | 110.8 | 82.8 | 356.6 | +203.6% | 10.3 | **N better** |
| N vs T | write_p99_us | 1174 | 437.4 | 2657 | 732.4 | 603.2 | 1185 | +126.3% | 2.46 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | commits_per_s | 1924 | 388.8 | 736.3 | 73.8 | 279.8 | 380.9 | -61.7% | 4.24 | **N better** |
| N vs T | pss_mib | 13.5 | 0.26 | 2526 | 0.01 | 0.19 | 1.17 | +18624.9% | 13472 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 9120 | 484.8 | -2383 | 80.7 | 347.5 | 3466 | -126.1% | 33.1 | **T better** |
| P+ vs T | q1_p50_us | 705.5 | 41.3 | 1349 | 107.2 | 81.2 | 188.1 | +91.3% | 7.93 | **P+ better** |
| P+ vs T | q1_p99_us | 2020 | 624.3 | 3022 | 554.3 | 590.4 | 896.7 | +49.6% | 1.70 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | q2_p50_us | 535.3 | 29.0 | 1391 | 95.2 | 70.4 | 247.9 | +159.9% | 12.2 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 894.6 | 110.3 | 1907 | 107.5 | 108.9 | 229.1 | +113.1% | 9.29 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 5821 | 798.6 | 5282 | 2042 | 1551 | 1698 | -9.3% | 0.35 | BELOW FLOOR |
| P+ vs T | c2_q2_p50_us | 717.1 | 74.5 | 1888 | 80.3 | 77.4 | 562.7 | +163.2% | 15.1 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 1974 | 62.7 | 2242 | 147.7 | 113.5 | 231.2 | +13.6% | 2.36 | no difference |
| P+ vs T | c4_q1_p99_us | 9849 | 3141 | 5092 | 1075 | 2347 | 21734 | -48.3% | 2.03 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q2_p50_us | 1652 | 67.9 | 2252 | 107.9 | 90.2 | 436.6 | +36.3% | 6.65 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 8449 | 1830 | 5526 | 861.1 | 1430 | 1440 | -34.6% | 2.04 | no difference |
| P+ vs T | write_p50_us | 945.5 | 36.7 | 1270 | 110.8 | 82.5 | 361.2 | +34.3% | 3.93 | BELOW FLOOR |
| P+ vs T | write_p99_us | 3262 | 249.6 | 2657 | 732.4 | 547.1 | 1313 | -18.6% | 1.11 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 916.8 | 10.3 | 736.3 | 73.8 | 52.7 | 240.4 | -19.7% | 3.43 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.02 | 2526 | 0.01 | 0.02 | 0.36 | +5738.7% | 150585 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -2692 | 85.9 | -2383 | 80.7 | 83.3 | 2010 | -11.5% | 3.71 | REFUSED (warm-up MAD above 15% of the median on P+) |
| H3 vs T | q1_p50_us | 303.9 | 30.3 | 1349 | 107.2 | 78.7 | 79.8 | +344.1% | 13.3 | **H3 better** |
| H3 vs T | q1_p99_us | 1258 | 319.9 | 3022 | 554.3 | 452.6 | 610.7 | +140.2% | 3.90 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q2_p50_us | 321.9 | 34.0 | 1391 | 95.2 | 71.5 | 244.4 | +332.2% | 15.0 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 327.2 | 38.2 | 1907 | 107.5 | 80.7 | 237.9 | +482.7% | 19.6 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 1175 | 320.0 | 5282 | 2042 | 1462 | 1357 | +349.6% | 2.81 | no difference |
| H3 vs T | c2_q2_p50_us | 351.5 | 25.5 | 1888 | 80.3 | 59.6 | 537.9 | +437.1% | 25.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p50_us | 450.8 | 20.4 | 2242 | 147.7 | 105.4 | 317.3 | +397.2% | 17.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 2084 | 359.0 | 5092 | 1075 | 801.2 | 999.8 | +144.4% | 3.75 | **H3 better** |
| H3 vs T | c4_q2_p50_us | 476.7 | 39.8 | 2252 | 107.9 | 81.3 | 439.1 | +372.5% | 21.8 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 1790 | 298.0 | 5526 | 861.1 | 644.4 | 1504 | +208.7% | 5.80 | **H3 better** |
| H3 vs T | write_p50_us | 1229 | 46.2 | 1270 | 110.8 | 84.9 | 345.1 | +3.3% | 0.48 | BELOW FLOOR |
| H3 vs T | write_p99_us | 6236 | 2724 | 2657 | 732.4 | 1994 | 1186 | -57.4% | 1.79 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 587.7 | 100.4 | 736.3 | 73.8 | 88.1 | 146.6 | +25.3% | 1.69 | no difference |
| H3 vs T | pss_mib | 67.6 | 0.09 | 2526 | 0.01 | 0.07 | 0.36 | +3636.9% | 36963 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 16039 | 322.0 | -2383 | 80.7 | 234.8 | 10072 | -114.9% | 78.5 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H1 | q1_p50_us | 111.0 | 13.1 | 834.6 | 23.5 | 19.0 | 83.7 | +651.6% | 38.0 | **N better** |
| N vs H1 | q1_p99_us | 360.4 | 68.3 | 2281 | 301.2 | 218.4 | 601.0 | +533.1% | 8.80 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p50_us | 124.1 | 13.0 | 705.3 | 19.7 | 16.7 | 74.0 | +468.1% | 34.8 | **N better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2125 | 83.3 | 550.0 | 112.7 | 99.1 | 162.0 | -74.1% | 15.9 | **H1 better** |
| N vs H1 | q6_p50_us | 2821 | 206.3 | 1187 | 159.8 | 184.5 | 618.3 | -57.9% | 8.86 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 121.2 | 27.9 | 883.0 | 59.7 | 46.6 | 191.5 | +628.8% | 16.4 | **N better** |
| N vs H1 | c2_q1_p99_us | 614.4 | 119.5 | 2507 | 611.1 | 440.3 | 146.6 | +308.0% | 4.30 | **N better** |
| N vs H1 | c2_q2_p50_us | 134.9 | 45.4 | 798.7 | 55.1 | 50.5 | 218.5 | +492.0% | 13.1 | **N better** |
| N vs H1 | c4_q1_p50_us | 220.2 | 60.0 | 902.3 | 44.4 | 52.8 | 91.5 | +309.7% | 12.9 | **N better** |
| N vs H1 | c4_q1_p99_us | 898.9 | 105.8 | 2147 | 349.2 | 258.0 | 1659 | +138.9% | 4.84 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | c4_q2_p50_us | 237.5 | 62.2 | 825.8 | 78.2 | 70.6 | 148.2 | +247.7% | 8.33 | **N better** |
| N vs H1 | c4_q2_p99_us | 1117 | 368.5 | 2722 | 314.4 | 342.5 | 554.5 | +143.7% | 4.69 | **N better** |
| N vs H1 | write_p50_us | 418.2 | 37.7 | 1445 | 34.8 | 36.3 | 208.6 | +245.4% | 28.3 | **N better** |
| N vs H1 | write_p99_us | 1174 | 437.4 | 5398 | 999.5 | 771.5 | 3614 | +359.8% | 5.48 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | commits_per_s | 1924 | 388.8 | 594.4 | 25.4 | 275.5 | 358.7 | -69.1% | 4.82 | **N better** |
| N vs H1 | pss_mib | 13.5 | 0.26 | 284.7 | 4.79 | 3.39 | 9.72 | +2010.4% | 80.0 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 9120 | 484.8 | 36008 | 3701 | 2639 | 6695 | +294.8% | 10.2 | **N better** |
| P+ vs H1 | q1_p50_us | 705.5 | 41.3 | 834.6 | 23.5 | 33.6 | 200.5 | +18.3% | 3.84 | BELOW FLOOR |
| P+ vs H1 | q1_p99_us | 2020 | 624.3 | 2281 | 301.2 | 490.1 | 809.4 | +12.9% | 0.53 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q2_p50_us | 535.3 | 29.0 | 705.3 | 19.7 | 24.8 | 87.0 | +31.7% | 6.85 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 271.9 | 47.6 | 776.1 | 53.4 | 50.5 | 115.0 | +185.5% | 9.98 | **P+ better** |
| P+ vs H1 | q4_p50_us | 1411 | 76.5 | 3189 | 231.2 | 172.2 | 452.4 | +126.0% | 10.3 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3210 | 156.8 | 550.0 | 112.7 | 136.5 | 642.1 | -82.9% | 19.5 | **H1 better** |
| P+ vs H1 | q6_p50_us | 3194 | 199.7 | 1187 | 159.8 | 180.8 | 678.4 | -62.8% | 11.1 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 894.6 | 110.3 | 883.0 | 59.7 | 88.7 | 192.3 | -1.3% | 0.13 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 5821 | 798.6 | 2507 | 611.1 | 711.1 | 1030 | -56.9% | 4.66 | **H1 better** |
| P+ vs H1 | c2_q2_p50_us | 717.1 | 74.5 | 798.7 | 55.1 | 65.5 | 331.9 | +11.4% | 1.24 | BELOW FLOOR |
| P+ vs H1 | c4_q1_p50_us | 1974 | 62.7 | 902.3 | 44.4 | 54.4 | 151.4 | -54.3% | 19.7 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 9849 | 3141 | 2147 | 349.2 | 2234 | 21744 | -78.2% | 3.45 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c4_q2_p50_us | 1652 | 67.9 | 825.8 | 78.2 | 73.2 | 175.5 | -50.0% | 11.3 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 8449 | 1830 | 2722 | 314.4 | 1313 | 172.6 | -67.8% | 4.36 | **H1 better** |
| P+ vs H1 | write_p50_us | 945.5 | 36.7 | 1445 | 34.8 | 35.7 | 216.3 | +52.8% | 14.0 | **P+ better** |
| P+ vs H1 | write_p99_us | 3262 | 249.6 | 5398 | 999.5 | 728.5 | 3658 | +65.5% | 2.93 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | commits_per_s | 916.8 | 10.3 | 594.4 | 25.4 | 19.4 | 203.4 | -35.2% | 16.6 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.02 | 284.7 | 4.79 | 3.38 | 9.66 | +558.0% | 71.3 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -2692 | 85.9 | 36008 | 3701 | 2617 | 6071 | -1437.8% | 14.8 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 111.0 | 13.1 | 83.6 | 10.5 | 11.9 | 6.51 | -24.7% | 2.31 | no difference |
| N vs H2 | q1_p99_us | 360.4 | 68.3 | 601.5 | 228.8 | 168.8 | 555.9 | +66.9% | 1.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 124.1 | 13.0 | 186.2 | 34.6 | 26.2 | 24.6 | +50.0% | 2.37 | no difference |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2125 | 83.3 | 115.8 | 19.0 | 60.4 | 165.3 | -94.6% | 33.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | 2821 | 206.3 | 508.3 | 54.6 | 150.9 | 557.8 | -82.0% | 15.3 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 121.2 | 27.9 | 91.5 | 23.9 | 26.0 | 32.5 | -24.5% | 1.14 | BELOW FLOOR |
| N vs H2 | c2_q1_p99_us | 614.4 | 119.5 | 1824 | 208.9 | 170.2 | 85.4 | +196.8% | 7.11 | **N better** |
| N vs H2 | c2_q2_p50_us | 134.9 | 45.4 | 176.6 | 51.7 | 48.6 | 80.3 | +30.9% | 0.86 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p50_us | 220.2 | 60.0 | 114.4 | 21.1 | 45.0 | 56.9 | -48.0% | 2.35 | no difference |
| N vs H2 | c4_q1_p99_us | 898.9 | 105.8 | 2008 | 193.9 | 156.2 | 1550 | +123.4% | 7.10 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 237.5 | 62.2 | 237.7 | 51.5 | 57.1 | 65.1 | +0.1% | 0.00 | BELOW FLOOR |
| N vs H2 | c4_q2_p99_us | 1117 | 368.5 | 2691 | 471.9 | 423.4 | 537.5 | +141.0% | 3.72 | **N better** |
| N vs H2 | write_p50_us | 418.2 | 37.7 | 1372 | 46.4 | 42.3 | 123.3 | +228.0% | 22.6 | **N better** |
| N vs H2 | write_p99_us | 1174 | 437.4 | 4909 | 1278 | 955.1 | 1320 | +318.1% | 3.91 | **N better** |
| N vs H2 | commits_per_s | 1924 | 388.8 | 608.1 | 33.3 | 275.9 | 359.4 | -68.4% | 4.77 | **N better** |
| N vs H2 | pss_mib | 13.5 | 0.26 | 43.4 | 0.01 | 0.19 | 1.12 | +221.9% | 160.5 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 9120 | 484.8 | -2581 | 87.4 | 348.3 | 3700 | -128.3% | 33.6 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p50_us | 705.5 | 41.3 | 83.6 | 10.5 | 30.1 | 182.3 | -88.2% | 20.6 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2020 | 624.3 | 601.5 | 228.8 | 470.2 | 776.6 | -70.2% | 3.02 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 535.3 | 29.0 | 186.2 | 34.6 | 31.9 | 52.0 | -65.2% | 10.9 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 271.9 | 47.6 | 165.1 | 31.5 | 40.3 | 112.0 | -39.3% | 2.65 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q4_p50_us | 1411 | 76.5 | 172.2 | 21.3 | 56.1 | 132.4 | -87.8% | 22.1 | **H2 better** |
| P+ vs H2 | q5_p50_us | 3210 | 156.8 | 115.8 | 19.0 | 111.7 | 642.9 | -96.4% | 27.7 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q6_p50_us | 3194 | 199.7 | 508.3 | 54.6 | 146.4 | 623.7 | -84.1% | 18.4 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 894.6 | 110.3 | 91.5 | 23.9 | 79.8 | 37.0 | -89.8% | 10.1 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5821 | 798.6 | 1824 | 208.9 | 583.7 | 1023 | -68.7% | 6.85 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 717.1 | 74.5 | 176.6 | 51.7 | 64.1 | 262.4 | -75.4% | 8.43 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p50_us | 1974 | 62.7 | 114.4 | 21.1 | 46.8 | 133.4 | -94.2% | 39.7 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 9849 | 3141 | 2008 | 193.9 | 2225 | 21736 | -79.6% | 3.52 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1652 | 67.9 | 237.7 | 51.5 | 60.3 | 114.3 | -85.6% | 23.5 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 8449 | 1830 | 2691 | 471.9 | 1336 | 105.9 | -68.1% | 4.31 | **H2 better** |
| P+ vs H2 | write_p50_us | 945.5 | 36.7 | 1372 | 46.4 | 41.8 | 135.9 | +45.1% | 10.2 | **P+ better** |
| P+ vs H2 | write_p99_us | 3262 | 249.6 | 4909 | 1278 | 920.7 | 1436 | +50.5% | 1.79 | no difference |
| P+ vs H2 | commits_per_s | 916.8 | 10.3 | 608.1 | 33.3 | 24.7 | 204.7 | -33.7% | 12.5 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.02 | 43.4 | 0.01 | 0.02 | 0.05 | +0.4% | 9.84 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -2692 | 85.9 | -2581 | 87.4 | 86.7 | 2391 | -4.1% | 1.27 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 129.0 | 16.5 | 675.4 | 12.9 | 14.8 | 0.53 | +423.7% | 36.9 | **N better** |
| N vs P+ | q1_p99_us | 457.6 | 166.3 | 2165 | 699.1 | 508.1 | 117.2 | +373.1% | 3.36 | **N better** |
| N vs P+ | q2_p50_us | 141.2 | 16.8 | 564.0 | 31.2 | 25.1 | 30.0 | +299.5% | 16.9 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2310 | 131.3 | 3259 | 171.7 | 152.9 | 554.9 | +41.1% | 6.21 | **N better** |
| N vs P+ | q6_p50_us | 3207 | 195.9 | 3705 | 215.0 | 205.7 | 369.5 | +15.5% | 2.42 | no difference |
| N vs P+ | c2_q1_p50_us | 149.5 | 34.2 | 908.8 | 114.3 | 84.4 | 68.6 | +508.0% | 9.00 | **N better** |
| N vs P+ | c2_q1_p99_us | 648.1 | 185.5 | 6025 | 2211 | 1569 | 2779 | +829.7% | 3.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 170.3 | 36.4 | 728.1 | 103.8 | 77.8 | 153.5 | +327.7% | 7.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 220.9 | 49.1 | 1923 | 140.6 | 105.3 | 292.4 | +770.4% | 16.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 965.9 | 165.6 | 10116 | 5075 | 3590 | 5624 | +947.3% | 2.55 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 254.2 | 72.1 | 1665 | 105.9 | 90.6 | 205.4 | +555.2% | 15.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1160 | 255.4 | 9020 | 2585 | 1836 | 11720 | +677.8% | 4.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 500.7 | 31.2 | 967.3 | 67.9 | 52.9 | 162.2 | +93.2% | 8.83 | **N better** |
| N vs P+ | write_p99_us | 1049 | 436.3 | 3200 | 532.4 | 486.7 | 322.1 | +204.9% | 4.42 | **N better** |
| N vs P+ | commits_per_s | 1864 | 292.5 | 882.1 | 62.4 | 211.5 | 327.7 | -52.7% | 4.64 | **N better** |
| N vs P+ | pss_mib | 13.0 | 0.54 | 43.3 | 0.01 | 0.38 | 1.53 | +231.7% | 79.9 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 8939 | 234.4 | -2567 | 106.1 | 182.0 | 11576 | -128.7% | 63.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 129.0 | 16.5 | 111.6 | 13.1 | 14.9 | 33.5 | -13.5% | 1.17 | BELOW FLOOR |
| N vs P | q1_p99_us | 457.6 | 166.3 | 667.2 | 166.7 | 166.5 | 234.4 | +45.8% | 1.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 141.2 | 16.8 | 189.6 | 16.6 | 16.7 | 14.4 | +34.3% | 2.90 | no difference |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2310 | 131.3 | 132.4 | 15.9 | 93.5 | 187.8 | -94.3% | 23.3 | **P better** |
| N vs P | q6_p50_us | 3207 | 195.9 | 592.5 | 43.7 | 141.9 | 168.0 | -81.5% | 18.4 | **P better** |
| N vs P | c2_q1_p50_us | 149.5 | 34.2 | 93.1 | 34.8 | 34.5 | 23.3 | -37.7% | 1.64 | no difference |
| N vs P | c2_q1_p99_us | 648.1 | 185.5 | 2262 | 668.4 | 490.5 | 2025 | +249.0% | 3.29 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 170.3 | 36.4 | 177.8 | 26.6 | 31.9 | 55.8 | +4.5% | 0.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 220.9 | 49.1 | 150.4 | 27.7 | 39.9 | 95.8 | -31.9% | 1.77 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 965.9 | 165.6 | 2121 | 536.9 | 397.3 | 480.0 | +119.6% | 2.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 254.2 | 72.1 | 234.5 | 48.6 | 61.5 | 107.4 | -7.7% | 0.32 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 1160 | 255.4 | 2639 | 417.4 | 346.0 | 615.0 | +127.5% | 4.27 | **N better** |
| N vs P | write_p50_us | 500.7 | 31.2 | 13260 | 278.9 | 198.4 | 1154 | +2548.3% | 64.3 | **N better** |
| N vs P | write_p99_us | 1049 | 436.3 | 19937 | 926.9 | 724.4 | 781.9 | +1799.8% | 26.1 | **N better** |
| N vs P | commits_per_s | 1864 | 292.5 | 71.6 | 1.36 | 206.8 | 314.5 | -96.2% | 8.67 | **N better** |
| N vs P | pss_mib | 13.0 | 0.54 | 43.4 | 0.03 | 0.38 | 1.53 | +232.3% | 79.9 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 8939 | 234.4 | -2388 | 105.8 | 181.9 | 11591 | -126.7% | 62.3 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 675.4 | 12.9 | 111.6 | 13.1 | 13.0 | 33.5 | -83.5% | 43.4 | **P better** |
| P+ vs P | q1_p99_us | 2165 | 699.1 | 667.2 | 166.7 | 508.2 | 216.5 | -69.2% | 2.95 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 564.0 | 31.2 | 189.6 | 16.6 | 25.0 | 28.0 | -66.4% | 15.0 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 303.1 | 27.5 | 230.3 | 12.9 | 21.5 | 95.7 | -24.0% | 3.39 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1384 | 87.6 | 187.4 | 14.3 | 62.7 | 194.8 | -86.5% | 19.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q5_p50_us | 3259 | 171.7 | 132.4 | 15.9 | 122.0 | 522.2 | -95.9% | 25.6 | **P better** |
| P+ vs P | q6_p50_us | 3705 | 215.0 | 592.5 | 43.7 | 155.1 | 357.0 | -84.0% | 20.1 | **P better** |
| P+ vs P | c2_q1_p50_us | 908.8 | 114.3 | 93.1 | 34.8 | 84.5 | 65.4 | -89.8% | 9.65 | **P better** |
| P+ vs P | c2_q1_p99_us | 6025 | 2211 | 2262 | 668.4 | 1633 | 3437 | -62.5% | 2.30 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 728.1 | 103.8 | 177.8 | 26.6 | 75.8 | 150.3 | -75.6% | 7.26 | **P better** |
| P+ vs P | c4_q1_p50_us | 1923 | 140.6 | 150.4 | 27.7 | 101.4 | 277.1 | -92.2% | 17.5 | **P better** |
| P+ vs P | c4_q1_p99_us | 10116 | 5075 | 2121 | 536.9 | 3609 | 5614 | -79.0% | 2.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1665 | 105.9 | 234.5 | 48.6 | 82.4 | 176.1 | -85.9% | 17.4 | **P better** |
| P+ vs P | c4_q2_p99_us | 9020 | 2585 | 2639 | 417.4 | 1851 | 11735 | -70.7% | 3.45 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 967.3 | 67.9 | 13260 | 278.9 | 202.9 | 1156 | +1270.8% | 60.6 | **P+ better** |
| P+ vs P | write_p99_us | 3200 | 532.4 | 19937 | 926.9 | 755.9 | 713.4 | +523.1% | 22.1 | **P+ better** |
| P+ vs P | commits_per_s | 882.1 | 62.4 | 71.6 | 1.36 | 44.1 | 92.2 | -91.9% | 18.4 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.01 | 43.4 | 0.03 | 0.02 | 0.14 | +0.2% | 3.37 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2567 | 106.1 | -2388 | 105.8 | 105.9 | 2815 | -7.0% | 1.69 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 111.6 | 13.1 | 108.5 | 10.3 | 11.8 | 34.3 | -2.8% | 0.26 | BELOW FLOOR |
| P vs M | q1_p99_us | 667.2 | 166.7 | 598.2 | 143.6 | 155.6 | 213.3 | -10.3% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 189.6 | 16.6 | 215.0 | 18.9 | 17.8 | 26.6 | +13.4% | 1.43 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 230.3 | 12.9 | 248.1 | 67.8 | 48.8 | 84.2 | +7.7% | 0.37 | BELOW FLOOR |
| P vs M | q4_p50_us | 187.4 | 14.3 | 218.5 | 36.6 | 27.8 | 118.8 | +16.6% | 1.12 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q5_p50_us | 132.4 | 15.9 | 129.0 | 25.3 | 21.1 | 21.9 | -2.6% | 0.16 | BELOW FLOOR |
| P vs M | q6_p50_us | 592.5 | 43.7 | 737.1 | 53.4 | 48.8 | 158.6 | +24.4% | 2.96 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 93.1 | 34.8 | 96.6 | 33.0 | 33.9 | 63.5 | +3.8% | 0.10 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 2262 | 668.4 | 1869 | 406.3 | 553.1 | 2607 | -17.3% | 0.71 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 177.8 | 26.6 | 213.1 | 59.3 | 45.9 | 69.0 | +19.8% | 0.77 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 150.4 | 27.7 | 109.0 | 9.99 | 20.9 | 94.4 | -27.5% | 1.99 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2121 | 536.9 | 2913 | 800.2 | 681.4 | 1139 | +37.3% | 1.16 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 234.5 | 48.6 | 220.6 | 14.5 | 35.9 | 159.8 | -5.9% | 0.39 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p99_us | 2639 | 417.4 | 2485 | 542.5 | 484.0 | 2824 | -5.8% | 0.32 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 13260 | 278.9 | 13646 | 632.2 | 488.6 | 1202 | +2.9% | 0.79 | BELOW FLOOR |
| P vs M | write_p99_us | 19937 | 926.9 | 21533 | 2526 | 1903 | 2327 | +8.0% | 0.84 | BELOW FLOOR |
| P vs M | commits_per_s | 71.6 | 1.36 | 68.4 | 2.78 | 2.19 | 9.15 | -4.4% | 1.43 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.03 | 43.3 | 0.00 | 0.02 | 0.09 | -0.2% | 3.09 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2388 | 105.8 | -2552 | 106.9 | 106.3 | 2685 | +6.9% | 1.55 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 129.0 | 16.5 | 303.6 | 47.9 | 35.9 | 73.8 | +135.4% | 4.87 | **N better** |
| N vs H3 | q1_p99_us | 457.6 | 166.3 | 978.0 | 264.8 | 221.1 | 1150 | +113.7% | 2.35 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 141.2 | 16.8 | 305.6 | 60.7 | 44.5 | 59.4 | +116.4% | 3.69 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 2310 | 131.3 | 3729 | 323.2 | 246.7 | 199.1 | +61.4% | 5.75 | **N better** |
| N vs H3 | q6_p50_us | 3207 | 195.9 | 4182 | 269.2 | 235.4 | 191.1 | +30.4% | 4.14 | **N better** |
| N vs H3 | c2_q1_p50_us | 149.5 | 34.2 | 300.8 | 45.8 | 40.4 | 24.2 | +101.2% | 3.75 | **N better** |
| N vs H3 | c2_q1_p99_us | 648.1 | 185.5 | 1779 | 555.3 | 414.0 | 6804 | +174.5% | 2.73 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 170.3 | 36.4 | 342.2 | 67.2 | 54.1 | 76.0 | +101.0% | 3.18 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p50_us | 220.9 | 49.1 | 554.7 | 59.9 | 54.8 | 148.8 | +151.1% | 6.09 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 965.9 | 165.6 | 1733 | 299.9 | 242.3 | 1774 | +79.4% | 3.17 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 254.2 | 72.1 | 574.9 | 47.8 | 61.2 | 135.8 | +126.2% | 5.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1160 | 255.4 | 1747 | 467.5 | 376.7 | 431.8 | +50.6% | 1.56 | no difference |
| N vs H3 | write_p50_us | 500.7 | 31.2 | 1321 | 126.7 | 92.3 | 110.7 | +163.9% | 8.89 | **N better** |
| N vs H3 | write_p99_us | 1049 | 436.3 | 4599 | 1075 | 820.3 | 5183 | +338.2% | 4.33 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | commits_per_s | 1864 | 292.5 | 659.9 | 107.9 | 220.4 | 315.3 | -64.6% | 5.46 | **N better** |
| N vs H3 | pss_mib | 13.0 | 0.54 | 67.6 | 0.19 | 0.40 | 1.60 | +418.3% | 136.2 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 8939 | 234.4 | 16228 | 417.6 | 338.7 | 14904 | +81.6% | 21.5 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 675.4 | 12.9 | 303.6 | 47.9 | 35.1 | 73.8 | -55.0% | 10.6 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2165 | 699.1 | 978.0 | 264.8 | 528.6 | 1146 | -54.8% | 2.25 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 564.0 | 31.2 | 305.6 | 60.7 | 48.2 | 64.1 | -45.8% | 5.36 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 303.1 | 27.5 | 330.9 | 43.5 | 36.4 | 71.7 | +9.2% | 0.76 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 1384 | 87.6 | 1458 | 153.2 | 124.8 | 228.6 | +5.3% | 0.59 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 3259 | 171.7 | 3729 | 323.2 | 258.8 | 526.3 | +14.4% | 1.81 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 3705 | 215.0 | 4182 | 269.2 | 243.6 | 368.4 | +12.9% | 1.96 | no difference |
| P+ vs H3 | c2_q1_p50_us | 908.8 | 114.3 | 300.8 | 45.8 | 87.1 | 65.7 | -66.9% | 6.98 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 6025 | 2211 | 1779 | 555.3 | 1612 | 7349 | -70.5% | 2.63 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 728.1 | 103.8 | 342.2 | 67.2 | 87.5 | 158.9 | -53.0% | 4.41 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1923 | 140.6 | 554.7 | 59.9 | 108.1 | 299.6 | -71.1% | 12.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 10116 | 5075 | 1733 | 299.9 | 3595 | 5868 | -82.9% | 2.33 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1665 | 105.9 | 574.9 | 47.8 | 82.1 | 194.7 | -65.5% | 13.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 9020 | 2585 | 1747 | 467.5 | 1857 | 11727 | -80.6% | 3.92 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 967.3 | 67.9 | 1321 | 126.7 | 101.7 | 129.4 | +36.6% | 3.48 | **P+ better** |
| P+ vs H3 | write_p99_us | 3200 | 532.4 | 4599 | 1075 | 848.2 | 5174 | +43.7% | 1.65 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 882.1 | 62.4 | 659.9 | 107.9 | 88.1 | 95.2 | -25.2% | 2.52 | no difference |
| P+ vs H3 | pss_mib | 43.3 | 0.01 | 67.6 | 0.19 | 0.13 | 0.50 | +56.3% | 185.0 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -2567 | 106.1 | 16228 | 417.6 | 304.7 | 9782 | -732.3% | 61.7 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 129.0 | 16.5 | 1219 | 114.3 | 81.6 | 181.3 | +844.8% | 13.3 | **N better** |
| N vs T | q1_p99_us | 457.6 | 166.3 | 3760 | 826.3 | 596.0 | 809.6 | +721.7% | 5.54 | **N better** |
| N vs T | q2_p50_us | 141.2 | 16.8 | 1149 | 103.4 | 74.1 | 126.9 | +713.6% | 13.6 | **N better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 149.5 | 34.2 | 1725 | 77.3 | 59.8 | 149.2 | +1054.4% | 26.4 | **N better** |
| N vs T | c2_q1_p99_us | 648.1 | 185.5 | 3857 | 714.3 | 521.8 | 1958 | +495.1% | 6.15 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c2_q2_p50_us | 170.3 | 36.4 | 1829 | 112.1 | 83.3 | 128.5 | +974.3% | 19.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p50_us | 220.9 | 49.1 | 2062 | 126.1 | 95.7 | 101.6 | +833.3% | 19.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 965.9 | 165.6 | 5834 | 1003 | 718.9 | 1636 | +504.0% | 6.77 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p50_us | 254.2 | 72.1 | 2084 | 206.2 | 154.5 | 131.6 | +719.7% | 11.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p99_us | 1160 | 255.4 | 5553 | 1584 | 1134 | 1627 | +378.9% | 3.87 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 500.7 | 31.2 | 1126 | 83.0 | 62.7 | 263.1 | +124.9% | 9.98 | **N better** |
| N vs T | write_p99_us | 1049 | 436.3 | 2831 | 998.5 | 770.5 | 696.0 | +169.7% | 2.31 | no difference |
| N vs T | commits_per_s | 1864 | 292.5 | 793.3 | 63.3 | 211.6 | 354.0 | -57.4% | 5.06 | **N better** |
| N vs T | pss_mib | 13.0 | 0.54 | 2523 | 0.02 | 0.38 | 2.21 | +19238.2% | 6630 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 8939 | 234.4 | -3358 | 144.0 | 194.5 | 11432 | -137.6% | 63.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs T | q1_p50_us | 675.4 | 12.9 | 1219 | 114.3 | 81.3 | 181.3 | +80.4% | 6.68 | **P+ better** |
| P+ vs T | q1_p99_us | 2165 | 699.1 | 3760 | 826.3 | 765.3 | 804.6 | +73.7% | 2.08 | no difference |
| P+ vs T | q2_p50_us | 564.0 | 31.2 | 1149 | 103.4 | 76.4 | 129.2 | +103.7% | 7.65 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 908.8 | 114.3 | 1725 | 77.3 | 97.6 | 161.3 | +89.9% | 8.37 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 6025 | 2211 | 3857 | 714.3 | 1643 | 3398 | -36.0% | 1.32 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c2_q2_p50_us | 728.1 | 103.8 | 1829 | 112.1 | 108.0 | 189.7 | +151.2% | 10.2 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 1923 | 140.6 | 2062 | 126.1 | 133.6 | 279.1 | +7.2% | 1.04 | BELOW FLOOR |
| P+ vs T | c4_q1_p99_us | 10116 | 5075 | 5834 | 1003 | 3658 | 5828 | -42.3% | 1.17 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q2_p50_us | 1665 | 105.9 | 2084 | 206.2 | 163.9 | 191.8 | +25.1% | 2.55 | no difference |
| P+ vs T | c4_q2_p99_us | 9020 | 2585 | 5553 | 1584 | 2143 | 11831 | -38.4% | 1.62 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | write_p50_us | 967.3 | 67.9 | 1126 | 83.0 | 75.8 | 271.4 | +16.4% | 2.10 | BELOW FLOOR |
| P+ vs T | write_p99_us | 3200 | 532.4 | 2831 | 998.5 | 800.1 | 618.1 | -11.5% | 0.46 | BELOW FLOOR |
| P+ vs T | commits_per_s | 882.1 | 62.4 | 793.3 | 63.3 | 62.9 | 186.9 | -10.1% | 1.41 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.01 | 2523 | 0.02 | 0.01 | 1.60 | +5730.9% | 187673 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -2567 | 106.1 | -3358 | 144.0 | 126.5 | 2065 | +30.8% | 6.26 | REFUSED (warm-up MAD above 15% of the median on P+) |
| H3 vs T | q1_p50_us | 303.6 | 47.9 | 1219 | 114.3 | 87.6 | 195.7 | +301.4% | 10.4 | **H3 better** |
| H3 vs T | q1_p99_us | 978.0 | 264.8 | 3760 | 826.3 | 613.5 | 1399 | +284.5% | 4.53 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q2_p50_us | 305.6 | 60.7 | 1149 | 103.4 | 84.8 | 139.0 | +275.9% | 9.94 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 300.8 | 45.8 | 1725 | 77.3 | 63.5 | 147.9 | +473.6% | 22.4 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 1779 | 555.3 | 3857 | 714.3 | 639.7 | 7079 | +116.8% | 3.25 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c2_q2_p50_us | 342.2 | 67.2 | 1829 | 112.1 | 92.4 | 134.9 | +434.5% | 16.1 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 554.7 | 59.9 | 2062 | 126.1 | 98.8 | 120.7 | +271.7% | 15.3 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 1733 | 299.9 | 5834 | 1003 | 740.4 | 2342 | +236.7% | 5.54 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p50_us | 574.9 | 47.8 | 2084 | 206.2 | 149.7 | 114.2 | +262.4% | 10.1 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 1747 | 467.5 | 5553 | 1584 | 1167 | 1677 | +217.9% | 3.26 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | write_p50_us | 1321 | 126.7 | 1126 | 83.0 | 107.1 | 244.2 | -14.8% | 1.82 | BELOW FLOOR |
| H3 vs T | write_p99_us | 4599 | 1075 | 2831 | 998.5 | 1037 | 5210 | -38.5% | 1.70 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | commits_per_s | 659.9 | 107.9 | 793.3 | 63.3 | 88.5 | 164.3 | +20.2% | 1.51 | BELOW FLOOR |
| H3 vs T | pss_mib | 67.6 | 0.19 | 2523 | 0.02 | 0.13 | 1.67 | +3631.3% | 18586 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 16228 | 417.6 | -3358 | 144.0 | 312.4 | 9612 | -120.7% | 62.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H1 | q1_p50_us | 129.0 | 16.5 | 869.2 | 61.0 | 44.7 | 76.6 | +573.9% | 16.6 | **N better** |
| N vs H1 | q1_p99_us | 457.6 | 166.3 | 2644 | 722.3 | 524.1 | 1462 | +477.8% | 4.17 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q2_p50_us | 141.2 | 16.8 | 738.1 | 87.6 | 63.1 | 89.4 | +422.8% | 9.46 | **N better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 2310 | 131.3 | 505.5 | 72.6 | 106.1 | 252.5 | -78.1% | 17.0 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | q6_p50_us | 3207 | 195.9 | 1412 | 145.7 | 172.6 | 241.6 | -56.0% | 10.4 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 149.5 | 34.2 | 974.7 | 36.9 | 35.6 | 135.9 | +552.2% | 23.2 | **N better** |
| N vs H1 | c2_q1_p99_us | 648.1 | 185.5 | 2462 | 173.7 | 179.7 | 1090 | +279.8% | 10.1 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c2_q2_p50_us | 170.3 | 36.4 | 841.1 | 72.2 | 57.1 | 146.9 | +394.0% | 11.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p50_us | 220.9 | 49.1 | 917.9 | 61.6 | 55.7 | 163.1 | +315.5% | 12.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 965.9 | 165.6 | 2566 | 291.2 | 236.9 | 449.7 | +165.7% | 6.75 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p50_us | 254.2 | 72.1 | 839.3 | 48.5 | 61.4 | 122.8 | +230.2% | 9.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p99_us | 1160 | 255.4 | 3154 | 479.7 | 384.3 | 826.8 | +172.0% | 5.19 | **N better** |
| N vs H1 | write_p50_us | 500.7 | 31.2 | 1522 | 72.4 | 55.8 | 137.4 | +204.1% | 18.3 | **N better** |
| N vs H1 | write_p99_us | 1049 | 436.3 | 4726 | 319.1 | 382.2 | 2312 | +350.3% | 9.62 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | commits_per_s | 1864 | 292.5 | 574.9 | 31.8 | 208.0 | 331.6 | -69.2% | 6.20 | **N better** |
| N vs H1 | pss_mib | 13.0 | 0.54 | 282.2 | 7.80 | 5.53 | 10.2 | +2062.6% | 48.7 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 8939 | 234.4 | 32386 | 6484 | 4588 | 11553 | +262.3% | 5.11 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H1 | q1_p50_us | 675.4 | 12.9 | 869.2 | 61.0 | 44.1 | 76.6 | +28.7% | 4.40 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2165 | 699.1 | 2644 | 722.3 | 710.8 | 1460 | +22.1% | 0.67 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q2_p50_us | 564.0 | 31.2 | 738.1 | 87.6 | 65.8 | 92.6 | +30.9% | 2.65 | no difference |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 303.1 | 27.5 | 783.2 | 117.4 | 85.3 | 132.5 | +158.4% | 5.63 | **P+ better** |
| P+ vs H1 | q4_p50_us | 1384 | 87.6 | 2958 | 234.1 | 176.8 | 656.1 | +113.7% | 8.90 | **P+ better** |
| P+ vs H1 | q5_p50_us | 3259 | 171.7 | 505.5 | 72.6 | 131.8 | 548.7 | -84.5% | 20.9 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q6_p50_us | 3705 | 215.0 | 1412 | 145.7 | 183.6 | 397.0 | -61.9% | 12.5 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 908.8 | 114.3 | 974.7 | 36.9 | 85.0 | 149.0 | +7.3% | 0.78 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 6025 | 2211 | 2462 | 173.7 | 1568 | 2983 | -59.1% | 2.27 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c2_q2_p50_us | 728.1 | 103.8 | 841.1 | 72.2 | 89.4 | 202.6 | +15.5% | 1.26 | BELOW FLOOR |
| P+ vs H1 | c4_q1_p50_us | 1923 | 140.6 | 917.9 | 61.6 | 108.6 | 306.9 | -52.3% | 9.26 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 10116 | 5075 | 2566 | 291.2 | 3594 | 5611 | -74.6% | 2.10 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q2_p50_us | 1665 | 105.9 | 839.3 | 48.5 | 82.3 | 185.9 | -49.6% | 10.0 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 9020 | 2585 | 3154 | 479.7 | 1859 | 11748 | -65.0% | 3.16 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | write_p50_us | 967.3 | 67.9 | 1522 | 72.4 | 70.2 | 152.8 | +57.4% | 7.91 | **P+ better** |
| P+ vs H1 | write_p99_us | 3200 | 532.4 | 4726 | 319.1 | 438.9 | 2290 | +47.7% | 3.48 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | commits_per_s | 882.1 | 62.4 | 574.9 | 31.8 | 49.5 | 140.0 | -34.8% | 6.21 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.01 | 282.2 | 7.80 | 5.51 | 10.1 | +552.1% | 43.3 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -2567 | 106.1 | 32386 | 6484 | 4586 | 2654 | -1361.8% | 7.62 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 129.0 | 16.5 | 103.3 | 15.4 | 16.0 | 1.02 | -19.9% | 1.61 | no difference |
| N vs H2 | q1_p99_us | 457.6 | 166.3 | 414.4 | 129.0 | 148.8 | 1185 | -9.4% | 0.29 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 141.2 | 16.8 | 202.7 | 54.8 | 40.5 | 51.8 | +43.5% | 1.52 | no difference |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 2310 | 131.3 | 131.8 | 20.8 | 94.0 | 188.0 | -94.3% | 23.2 | **H2 better** |
| N vs H2 | q6_p50_us | 3207 | 195.9 | 503.7 | 55.1 | 143.9 | 136.8 | -84.3% | 18.8 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 149.5 | 34.2 | 98.7 | 22.1 | 28.8 | 30.1 | -33.9% | 1.76 | no difference |
| N vs H2 | c2_q1_p99_us | 648.1 | 185.5 | 1596 | 390.1 | 305.4 | 782.3 | +146.2% | 3.10 | **N better** |
| N vs H2 | c2_q2_p50_us | 170.3 | 36.4 | 186.3 | 34.4 | 35.4 | 46.7 | +9.4% | 0.45 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p50_us | 220.9 | 49.1 | 151.0 | 14.7 | 36.2 | 106.1 | -31.6% | 1.93 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p99_us | 965.9 | 165.6 | 1848 | 471.5 | 353.4 | 591.1 | +91.3% | 2.49 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 254.2 | 72.1 | 262.7 | 39.2 | 58.1 | 158.7 | +3.3% | 0.15 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q2_p99_us | 1160 | 255.4 | 2935 | 1043 | 759.6 | 432.9 | +153.1% | 2.34 | no difference |
| N vs H2 | write_p50_us | 500.7 | 31.2 | 1473 | 187.3 | 134.3 | 309.1 | +194.1% | 7.24 | **N better** |
| N vs H2 | write_p99_us | 1049 | 436.3 | 4831 | 794.7 | 641.1 | 1152 | +360.4% | 5.90 | **N better** |
| N vs H2 | commits_per_s | 1864 | 292.5 | 572.0 | 51.0 | 209.9 | 318.0 | -69.3% | 6.15 | **N better** |
| N vs H2 | pss_mib | 13.0 | 0.54 | 43.6 | 0.01 | 0.38 | 1.52 | +233.9% | 80.7 | **N better** |
| N vs H2 | pss_growth_bytes_per_key_read | 8939 | 234.4 | -2612 | 107.5 | 182.4 | 11606 | -129.2% | 63.3 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 675.4 | 12.9 | 103.3 | 15.4 | 14.2 | 0.97 | -84.7% | 40.3 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2165 | 699.1 | 414.4 | 129.0 | 502.7 | 1182 | -80.9% | 3.48 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p50_us | 564.0 | 31.2 | 202.7 | 54.8 | 44.6 | 57.2 | -64.1% | 8.10 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 303.1 | 27.5 | 219.4 | 23.2 | 25.5 | 71.6 | -27.6% | 3.29 | **H2 better** |
| P+ vs H2 | q4_p50_us | 1384 | 87.6 | 216.9 | 34.7 | 66.6 | 178.4 | -84.3% | 17.5 | **H2 better** |
| P+ vs H2 | q5_p50_us | 3259 | 171.7 | 131.8 | 20.8 | 122.3 | 522.2 | -96.0% | 25.6 | **H2 better** |
| P+ vs H2 | q6_p50_us | 3705 | 215.0 | 503.7 | 55.1 | 156.9 | 343.4 | -86.4% | 20.4 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 908.8 | 114.3 | 98.7 | 22.1 | 82.3 | 68.1 | -89.1% | 9.84 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 6025 | 2211 | 1596 | 390.1 | 1588 | 2885 | -73.5% | 2.79 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q2_p50_us | 728.1 | 103.8 | 186.3 | 34.4 | 77.3 | 147.2 | -74.4% | 7.01 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1923 | 140.6 | 151.0 | 14.7 | 100.0 | 280.8 | -92.1% | 17.7 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 10116 | 5075 | 1848 | 471.5 | 3604 | 5624 | -81.7% | 2.29 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1665 | 105.9 | 262.7 | 39.2 | 79.8 | 211.3 | -84.2% | 17.6 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p99_us | 9020 | 2585 | 2935 | 1043 | 1971 | 11727 | -67.5% | 3.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 967.3 | 67.9 | 1473 | 187.3 | 140.9 | 316.3 | +52.2% | 3.59 | **P+ better** |
| P+ vs H2 | write_p99_us | 3200 | 532.4 | 4831 | 794.7 | 676.4 | 1107 | +51.0% | 2.41 | no difference |
| P+ vs H2 | commits_per_s | 882.1 | 62.4 | 572.0 | 51.0 | 57.0 | 103.6 | -35.2% | 5.44 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.01 | 43.6 | 0.01 | 0.01 | 0.11 | +0.7% | 36.0 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -2567 | 106.1 | -2612 | 107.5 | 106.8 | 2878 | +1.8% | 0.43 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 188.7 | 20.2 | 721.4 | 53.5 | 40.5 | 47.3 | +282.3% | 13.2 | **N better** |
| N vs P+ | q1_p99_us | 669.1 | 108.0 | 2520 | 522.3 | 377.1 | 1375 | +276.6% | 4.91 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 156.5 | 13.8 | 578.2 | 32.0 | 24.6 | 92.3 | +269.5% | 17.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 28770 | 572.9 | 42873 | 2784 | 2010 | 2388 | +49.0% | 7.02 | **N better** |
| N vs P+ | q6_p50_us | 46722 | 1965 | 41883 | 3008 | 2540 | 7460 | -10.4% | 1.90 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 221.4 | 45.8 | 933.9 | 118.7 | 90.0 | 660.2 | +321.8% | 7.92 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q1_p99_us | 1131 | 361.6 | 7591 | 1386 | 1013 | 6003 | +571.0% | 6.38 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 188.1 | 54.7 | 747.9 | 84.7 | 71.3 | 745.1 | +297.5% | 7.85 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q1_p50_us | 329.3 | 60.6 | 2008 | 88.0 | 75.5 | 382.2 | +509.8% | 22.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1410 | 258.1 | 10066 | 1854 | 1324 | 1398 | +614.0% | 6.54 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 250.7 | 47.7 | 1838 | 238.2 | 171.8 | 141.7 | +633.1% | 9.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 2062 | 567.2 | 7339 | 3018 | 2172 | 6106 | +255.9% | 2.43 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 533.8 | 28.8 | 962.0 | 53.8 | 43.1 | 180.7 | +80.2% | 9.93 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 1418 | 502.8 | 3297 | 934.6 | 750.4 | 721.2 | +132.4% | 2.50 | no difference |
| N vs P+ | commits_per_s | 1663 | 136.0 | 851.4 | 110.3 | 123.8 | 180.2 | -48.8% | 6.55 | **N better** |
| N vs P+ | pss_mib | 88.2 | 0.43 | 43.3 | 0.03 | 0.31 | 4.84 | -50.9% | 146.6 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 19475 | 4184 | -786.9 | 171.6 | 2961 | 107247 | -104.0% | 6.84 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 188.7 | 20.2 | 101.1 | 14.8 | 17.7 | 18.0 | -46.4% | 4.95 | **P better** |
| N vs P | q1_p99_us | 669.1 | 108.0 | 595.2 | 116.8 | 112.5 | 57.2 | -11.1% | 0.66 | no difference |
| N vs P | q2_p50_us | 156.5 | 13.8 | 196.8 | 14.3 | 14.0 | 61.9 | +25.7% | 2.87 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 28770 | 572.9 | 142.7 | 20.1 | 405.4 | 559.8 | -99.5% | 70.6 | **P better** |
| N vs P | q6_p50_us | 46722 | 1965 | 3248 | 471.4 | 1429 | 1325 | -93.0% | 30.4 | **P better** |
| N vs P | c2_q1_p50_us | 221.4 | 45.8 | 92.2 | 16.8 | 34.5 | 156.9 | -58.4% | 3.75 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 1131 | 361.6 | 2239 | 379.7 | 370.8 | 569.8 | +97.9% | 2.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 188.1 | 54.7 | 241.4 | 45.4 | 50.2 | 171.1 | +28.3% | 1.06 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 329.3 | 60.6 | 131.3 | 19.5 | 45.0 | 223.2 | -60.1% | 4.40 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1410 | 258.1 | 2670 | 168.6 | 218.0 | 1042 | +89.4% | 5.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 250.7 | 47.7 | 277.9 | 16.3 | 35.6 | 162.6 | +10.9% | 0.76 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 2062 | 567.2 | 3988 | 970.7 | 794.9 | 2853 | +93.4% | 2.42 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 533.8 | 28.8 | 120157 | 1374 | 972.0 | 8071 | +22410.2% | 123.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p99_us | 1418 | 502.8 | 159998 | 19872 | 14056 | 21301 | +11180.6% | 11.3 | **N better** |
| N vs P | commits_per_s | 1663 | 136.0 | 8.23 | 0.28 | 96.1 | 162.9 | -99.5% | 17.2 | **N better** |
| N vs P | pss_mib | 88.2 | 0.43 | 43.5 | 0.03 | 0.31 | 4.84 | -50.6% | 146.1 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 19475 | 4184 | -716.9 | 155.2 | 2960 | 107239 | -103.7% | 6.82 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 721.4 | 53.5 | 101.1 | 14.8 | 39.3 | 44.6 | -86.0% | 15.8 | **P better** |
| P+ vs P | q1_p99_us | 2520 | 522.3 | 595.2 | 116.8 | 378.4 | 1375 | -76.4% | 5.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 578.2 | 32.0 | 196.8 | 14.3 | 24.8 | 94.7 | -66.0% | 15.4 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 306.9 | 37.8 | 213.1 | 34.7 | 36.3 | 49.0 | -30.6% | 2.59 | no difference |
| P+ vs P | q4_p50_us | 12332 | 784.6 | 744.9 | 96.5 | 559.0 | 2196 | -94.0% | 20.7 | **P better** |
| P+ vs P | q5_p50_us | 42873 | 2784 | 142.7 | 20.1 | 1969 | 2322 | -99.7% | 21.7 | **P better** |
| P+ vs P | q6_p50_us | 41883 | 3008 | 3248 | 471.4 | 2153 | 7343 | -92.2% | 17.9 | **P better** |
| P+ vs P | c2_q1_p50_us | 933.9 | 118.7 | 92.2 | 16.8 | 84.8 | 644.8 | -90.1% | 9.93 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q1_p99_us | 7591 | 1386 | 2239 | 379.7 | 1016 | 6001 | -70.5% | 5.27 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 747.9 | 84.7 | 241.4 | 45.4 | 68.0 | 726.4 | -67.7% | 7.45 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p50_us | 2008 | 88.0 | 131.3 | 19.5 | 63.7 | 314.6 | -93.5% | 29.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 10066 | 1854 | 2670 | 168.6 | 1316 | 931.7 | -73.5% | 5.62 | **P better** |
| P+ vs P | c4_q2_p50_us | 1838 | 238.2 | 277.9 | 16.3 | 168.8 | 102.6 | -84.9% | 9.24 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 7339 | 3018 | 3988 | 970.7 | 2242 | 6588 | -45.7% | 1.49 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 962.0 | 53.8 | 120157 | 1374 | 972.5 | 8070 | +12390.2% | 122.6 | **P+ better** |
| P+ vs P | write_p99_us | 3297 | 934.6 | 159998 | 19872 | 14067 | 21308 | +4753.4% | 11.1 | **P+ better** |
| P+ vs P | commits_per_s | 851.4 | 110.3 | 8.23 | 0.28 | 78.0 | 76.8 | -99.0% | 10.8 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.03 | 43.5 | 0.03 | 0.03 | 0.20 | +0.5% | 6.69 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -786.9 | 171.6 | -716.9 | 155.2 | 163.6 | 2275 | -8.9% | 0.43 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 101.1 | 14.8 | 88.4 | 9.40 | 12.4 | 20.9 | -12.5% | 1.02 | BELOW FLOOR |
| P vs M | q1_p99_us | 595.2 | 116.8 | 609.0 | 178.3 | 150.7 | 365.8 | +2.3% | 0.09 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 196.8 | 14.3 | 212.6 | 26.1 | 21.0 | 88.1 | +8.0% | 0.75 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 213.1 | 34.7 | 214.0 | 56.7 | 47.0 | 41.4 | +0.4% | 0.02 | BELOW FLOOR |
| P vs M | q4_p50_us | 744.9 | 96.5 | 759.5 | 98.1 | 97.3 | 158.3 | +2.0% | 0.15 | BELOW FLOOR |
| P vs M | q5_p50_us | 142.7 | 20.1 | 129.5 | 10.5 | 16.0 | 106.1 | -9.2% | 0.82 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | 3248 | 471.4 | 3340 | 414.7 | 444.0 | 549.1 | +2.8% | 0.21 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 92.2 | 16.8 | 81.6 | 50.3 | 37.5 | 64.3 | -11.6% | 0.28 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 2239 | 379.7 | 1347 | 167.9 | 293.6 | 506.8 | -39.8% | 3.04 | **M better** |
| P vs M | c2_q2_p50_us | 241.4 | 45.4 | 213.9 | 63.4 | 55.1 | 33.6 | -11.4% | 0.50 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 131.3 | 19.5 | 138.7 | 24.3 | 22.0 | 69.3 | +5.6% | 0.33 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2670 | 168.6 | 2559 | 284.3 | 233.7 | 714.9 | -4.2% | 0.48 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 277.9 | 16.3 | 317.0 | 42.2 | 32.0 | 110.3 | +14.1% | 1.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 3988 | 970.7 | 3336 | 760.7 | 872.0 | 2680 | -16.3% | 0.75 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 120157 | 1374 | 120976 | 3483 | 2648 | 8097 | +0.7% | 0.31 | BELOW FLOOR |
| P vs M | write_p99_us | 159998 | 19872 | 161540 | 19138 | 19509 | 24254 | +1.0% | 0.08 | BELOW FLOOR |
| P vs M | commits_per_s | 8.23 | 0.28 | 8.11 | 0.50 | 0.41 | 1.02 | -1.4% | 0.29 | BELOW FLOOR |
| P vs M | pss_mib | 43.5 | 0.03 | 43.4 | 0.02 | 0.02 | 0.19 | -0.2% | 4.29 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -716.9 | 155.2 | -668.7 | 116.0 | 137.0 | 2134 | -6.7% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 188.7 | 20.2 | 406.3 | 12.3 | 16.7 | 39.4 | +115.3% | 13.0 | **N better** |
| N vs H3 | q1_p99_us | 669.1 | 108.0 | 1720 | 274.1 | 208.3 | 39.6 | +157.1% | 5.05 | **N better** |
| N vs H3 | q2_p50_us | 156.5 | 13.8 | 413.1 | 46.7 | 34.4 | 60.7 | +164.0% | 7.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 28770 | 572.9 | 38956 | 1879 | 1389 | 1104 | +35.4% | 7.33 | **N better** |
| N vs H3 | q6_p50_us | 46722 | 1965 | 45301 | 1419 | 1714 | 11056 | -3.0% | 0.83 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 221.4 | 45.8 | 409.1 | 56.1 | 51.2 | 165.3 | +84.8% | 3.67 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 1131 | 361.6 | 1604 | 482.3 | 426.2 | 423.8 | +41.8% | 1.11 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 188.1 | 54.7 | 432.6 | 61.0 | 57.9 | 240.8 | +130.0% | 4.22 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p50_us | 329.3 | 60.6 | 638.8 | 9.82 | 43.4 | 295.8 | +94.0% | 7.13 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 1410 | 258.1 | 3199 | 390.8 | 331.1 | 2265 | +126.9% | 5.40 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 250.7 | 47.7 | 657.9 | 59.5 | 53.9 | 187.5 | +162.5% | 7.56 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 2062 | 567.2 | 4580 | 1689 | 1260 | 2843 | +122.1% | 2.00 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 533.8 | 28.8 | 1304 | 90.4 | 67.1 | 225.0 | +144.3% | 11.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p99_us | 1418 | 502.8 | 4090 | 432.8 | 469.1 | 2228 | +188.4% | 5.70 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | commits_per_s | 1663 | 136.0 | 635.3 | 32.6 | 98.9 | 169.4 | -61.8% | 10.4 | **N better** |
| N vs H3 | pss_mib | 88.2 | 0.43 | 106.0 | 0.15 | 0.32 | 4.84 | +20.1% | 54.8 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 19475 | 4184 | 19583 | 3920 | 4054 | 115625 | +0.6% | 0.03 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 721.4 | 53.5 | 406.3 | 12.3 | 38.8 | 56.7 | -43.7% | 8.11 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2520 | 522.3 | 1720 | 274.1 | 417.1 | 1374 | -31.7% | 1.92 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 578.2 | 32.0 | 413.1 | 46.7 | 40.0 | 94.0 | -28.5% | 4.13 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 306.9 | 37.8 | 341.2 | 38.8 | 38.3 | 40.4 | +11.2% | 0.90 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 12332 | 784.6 | 12255 | 745.9 | 765.5 | 2435 | -0.6% | 0.10 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 42873 | 2784 | 38956 | 1879 | 2375 | 2509 | -9.1% | 1.65 | no difference |
| P+ vs H3 | q6_p50_us | 41883 | 3008 | 45301 | 1419 | 2352 | 13206 | +8.2% | 1.45 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 933.9 | 118.7 | 409.1 | 56.1 | 92.8 | 646.9 | -56.2% | 5.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q1_p99_us | 7591 | 1386 | 1604 | 482.3 | 1037 | 5989 | -78.9% | 5.77 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q2_p50_us | 747.9 | 84.7 | 432.6 | 61.0 | 73.8 | 745.9 | -42.2% | 4.27 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q1_p50_us | 2008 | 88.0 | 638.8 | 9.82 | 62.6 | 369.7 | -68.2% | 21.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 10066 | 1854 | 3199 | 390.8 | 1340 | 2217 | -68.2% | 5.13 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1838 | 238.2 | 657.9 | 59.5 | 173.6 | 138.6 | -64.2% | 6.79 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7339 | 3018 | 4580 | 1689 | 2446 | 6584 | -37.6% | 1.13 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 962.0 | 53.8 | 1304 | 90.4 | 74.4 | 186.1 | +35.5% | 4.60 | **P+ better** |
| P+ vs H3 | write_p99_us | 3297 | 934.6 | 4090 | 432.8 | 728.3 | 2302 | +24.1% | 1.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 851.4 | 110.3 | 635.3 | 32.6 | 81.3 | 89.7 | -25.4% | 2.66 | no difference |
| P+ vs H3 | pss_mib | 43.3 | 0.03 | 106.0 | 0.15 | 0.11 | 0.19 | +144.6% | 565.4 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -786.9 | 171.6 | 19583 | 3920 | 2775 | 43290 | -2588.5% | 7.34 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 188.7 | 20.2 | 1287 | 59.8 | 44.6 | 273.3 | +582.2% | 24.6 | **N better** |
| N vs T | q1_p99_us | 669.1 | 108.0 | 3289 | 749.4 | 535.4 | 11333 | +391.5% | 4.89 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | q2_p50_us | 156.5 | 13.8 | 1353 | 58.2 | 42.3 | 68.2 | +764.6% | 28.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 221.4 | 45.8 | 1935 | 132.5 | 99.1 | 340.5 | +774.0% | 17.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q1_p99_us | 1131 | 361.6 | 13521 | 7607 | 5385 | 16394 | +1095.2% | 2.30 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c2_q2_p50_us | 188.1 | 54.7 | 2007 | 237.3 | 172.2 | 241.4 | +967.1% | 10.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p50_us | 329.3 | 60.6 | 2391 | 176.1 | 131.6 | 391.4 | +626.0% | 15.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 1410 | 258.1 | 7389 | 3280 | 2326 | 1197 | +424.1% | 2.57 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p50_us | 250.7 | 47.7 | 2486 | 291.5 | 208.9 | 230.8 | +891.8% | 10.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p99_us | 2062 | 567.2 | 5090 | 1225 | 954.4 | 1381 | +146.8% | 3.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | write_p50_us | 533.8 | 28.8 | 1235 | 70.4 | 53.8 | 189.2 | +131.4% | 13.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | write_p99_us | 1418 | 502.8 | 2581 | 547.8 | 525.8 | 2125 | +82.0% | 2.21 | BELOW FLOOR |
| N vs T | commits_per_s | 1663 | 136.0 | 718.0 | 53.5 | 103.3 | 215.7 | -56.8% | 9.14 | **N better** |
| N vs T | pss_mib | 88.2 | 0.43 | 2582 | 0.10 | 0.31 | 4.84 | +2826.1% | 7976 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 19475 | 4184 | -323.8 | 109.5 | 2959 | 107238 | -101.7% | 6.69 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 721.4 | 53.5 | 1287 | 59.8 | 56.8 | 276.3 | +78.5% | 9.97 | **P+ better** |
| P+ vs T | q1_p99_us | 2520 | 522.3 | 3289 | 749.4 | 645.9 | 11416 | +30.5% | 1.19 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | q2_p50_us | 578.2 | 32.0 | 1353 | 58.2 | 47.0 | 99.0 | +134.0% | 16.5 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 933.9 | 118.7 | 1935 | 132.5 | 125.8 | 712.1 | +107.2% | 7.96 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c2_q1_p99_us | 7591 | 1386 | 13521 | 7607 | 5467 | 17449 | +78.1% | 1.08 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c2_q2_p50_us | 747.9 | 84.7 | 2007 | 237.3 | 178.1 | 746.1 | +168.4% | 7.07 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q1_p50_us | 2008 | 88.0 | 2391 | 176.1 | 139.2 | 449.9 | +19.1% | 2.75 | BELOW FLOOR |
| P+ vs T | c4_q1_p99_us | 10066 | 1854 | 7389 | 3280 | 2664 | 1103 | -26.6% | 1.00 | no difference |
| P+ vs T | c4_q2_p50_us | 1838 | 238.2 | 2486 | 291.5 | 266.2 | 193.3 | +35.3% | 2.44 | no difference |
| P+ vs T | c4_q2_p99_us | 7339 | 3018 | 5090 | 1225 | 2303 | 6097 | -30.6% | 0.98 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | write_p50_us | 962.0 | 53.8 | 1235 | 70.4 | 62.6 | 140.7 | +28.4% | 4.36 | **P+ better** |
| P+ vs T | write_p99_us | 3297 | 934.6 | 2581 | 547.8 | 766.0 | 2202 | -21.7% | 0.93 | BELOW FLOOR |
| P+ vs T | commits_per_s | 851.4 | 110.3 | 718.0 | 53.5 | 86.7 | 160.9 | -15.7% | 1.54 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.03 | 2582 | 0.10 | 0.07 | 0.23 | +5858.2% | 35267 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -786.9 | 171.6 | -323.8 | 109.5 | 144.0 | 2224 | -58.8% | 3.22 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 406.3 | 12.3 | 1287 | 59.8 | 43.2 | 275.0 | +216.9% | 20.4 | **H3 better** |
| H3 vs T | q1_p99_us | 1720 | 274.1 | 3289 | 749.4 | 564.3 | 11333 | +91.2% | 2.78 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | q2_p50_us | 413.1 | 46.7 | 1353 | 58.2 | 52.8 | 70.5 | +227.5% | 17.8 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 409.1 | 56.1 | 1935 | 132.5 | 101.7 | 313.8 | +373.0% | 15.0 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 1604 | 482.3 | 13521 | 7607 | 5389 | 16389 | +742.9% | 2.21 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c2_q2_p50_us | 432.6 | 61.0 | 2007 | 237.3 | 173.2 | 243.8 | +364.0% | 9.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p50_us | 638.8 | 9.82 | 2391 | 176.1 | 124.7 | 379.2 | +274.3% | 14.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 3199 | 390.8 | 7389 | 3280 | 2335 | 2096 | +131.0% | 1.79 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p50_us | 657.9 | 59.5 | 2486 | 291.5 | 210.4 | 229.0 | +277.9% | 8.69 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 4580 | 1689 | 5090 | 1225 | 1475 | 2822 | +11.1% | 0.35 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | write_p50_us | 1304 | 90.4 | 1235 | 70.4 | 81.0 | 194.4 | -5.3% | 0.85 | BELOW FLOOR |
| H3 vs T | write_p99_us | 4090 | 432.8 | 2581 | 547.8 | 493.7 | 3048 | -36.9% | 3.06 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | commits_per_s | 635.3 | 32.6 | 718.0 | 53.5 | 44.3 | 148.7 | +13.0% | 1.87 | BELOW FLOOR |
| H3 vs T | pss_mib | 106.0 | 0.15 | 2582 | 0.10 | 0.13 | 0.28 | +2335.8% | 19416 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 19583 | 3920 | -323.8 | 109.5 | 2773 | 43268 | -101.7% | 7.18 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 188.7 | 20.2 | 978.0 | 13.8 | 17.3 | 34.7 | +418.3% | 45.7 | **N better** |
| N vs H1 | q1_p99_us | 669.1 | 108.0 | 5270 | 999.5 | 710.9 | 204.9 | +687.6% | 6.47 | **N better** |
| N vs H1 | q2_p50_us | 156.5 | 13.8 | 788.5 | 25.9 | 20.8 | 55.8 | +403.9% | 30.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 28770 | 572.9 | 845.9 | 441.7 | 511.6 | 69896 | -97.1% | 54.6 | BELOW FLOOR |
| N vs H1 | q6_p50_us | 46722 | 1965 | 6853 | 735.9 | 1484 | 2833 | -85.3% | 26.9 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c2_q1_p50_us | 221.4 | 45.8 | 1121 | 72.3 | 60.5 | 286.1 | +406.4% | 14.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q1_p99_us | 1131 | 361.6 | 5799 | 999.0 | 751.2 | 2748 | +412.6% | 6.21 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | c2_q2_p50_us | 188.1 | 54.7 | 963.6 | 61.8 | 58.3 | 252.5 | +412.2% | 13.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p50_us | 329.3 | 60.6 | 1049 | 23.7 | 46.0 | 264.9 | +218.6% | 15.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 1410 | 258.1 | 4982 | 551.4 | 430.5 | 1096 | +253.4% | 8.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p50_us | 250.7 | 47.7 | 966.8 | 49.2 | 48.4 | 148.4 | +285.7% | 14.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p99_us | 2062 | 567.2 | 4164 | 1024 | 827.5 | 2667 | +101.9% | 2.54 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | write_p50_us | 533.8 | 28.8 | 1600 | 74.5 | 56.5 | 224.4 | +199.8% | 18.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | write_p99_us | 1418 | 502.8 | 7021 | 2214 | 1606 | 492.2 | +395.0% | 3.49 | **N better** |
| N vs H1 | commits_per_s | 1663 | 136.0 | 506.7 | 31.8 | 98.7 | 178.1 | -69.5% | 11.7 | **N better** |
| N vs H1 | pss_mib | 88.2 | 0.43 | 426.6 | 5.46 | 3.87 | 18.2 | +383.5% | 87.4 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 19475 | 4184 | 15059 | 1527 | 3149 | 107254 | -22.7% | 1.40 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H1 | q1_p50_us | 721.4 | 53.5 | 978.0 | 13.8 | 39.1 | 53.5 | +35.6% | 6.57 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2520 | 522.3 | 5270 | 999.5 | 797.4 | 1389 | +109.2% | 3.45 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q2_p50_us | 578.2 | 32.0 | 788.5 | 25.9 | 29.1 | 90.9 | +36.4% | 7.22 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 306.9 | 37.8 | 871.5 | 70.5 | 56.5 | 267.2 | +184.0% | 9.99 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q4_p50_us | 12332 | 784.6 | 26444 | 1450 | 1166 | 2516 | +114.4% | 12.1 | **P+ better** |
| P+ vs H1 | q5_p50_us | 42873 | 2784 | 845.9 | 441.7 | 1993 | 69933 | -98.0% | 21.1 | BELOW FLOOR |
| P+ vs H1 | q6_p50_us | 41883 | 3008 | 6853 | 735.9 | 2190 | 7758 | -83.6% | 16.0 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | c2_q1_p50_us | 933.9 | 118.7 | 1121 | 72.3 | 98.3 | 687.8 | +20.1% | 1.91 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c2_q1_p99_us | 7591 | 1386 | 5799 | 999.0 | 1208 | 6575 | -23.6% | 1.48 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c2_q2_p50_us | 747.9 | 84.7 | 963.6 | 61.8 | 74.2 | 749.8 | +28.8% | 2.91 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q1_p50_us | 2008 | 88.0 | 1049 | 23.7 | 64.4 | 345.4 | -47.7% | 14.9 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 10066 | 1854 | 4982 | 551.4 | 1368 | 991.1 | -50.5% | 3.72 | **H1 better** |
| P+ vs H1 | c4_q2_p50_us | 1838 | 238.2 | 966.8 | 49.2 | 172.0 | 78.1 | -47.4% | 5.06 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 7339 | 3018 | 4164 | 1024 | 2254 | 6510 | -43.3% | 1.41 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | write_p50_us | 962.0 | 53.8 | 1600 | 74.5 | 65.0 | 185.3 | +66.3% | 9.83 | **P+ better** |
| P+ vs H1 | write_p99_us | 3297 | 934.6 | 7021 | 2214 | 1700 | 758.6 | +113.0% | 2.19 | no difference |
| P+ vs H1 | commits_per_s | 851.4 | 110.3 | 506.7 | 31.8 | 81.1 | 105.2 | -40.5% | 4.25 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.03 | 426.6 | 5.46 | 3.86 | 17.5 | +884.5% | 99.3 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -786.9 | 171.6 | 15059 | 1527 | 1086 | 2888 | -2013.7% | 14.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 188.7 | 20.2 | 104.4 | 12.5 | 16.8 | 41.9 | -44.7% | 5.02 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q1_p99_us | 669.1 | 108.0 | 497.4 | 66.4 | 89.7 | 50.2 | -25.7% | 1.91 | no difference |
| N vs H2 | q2_p50_us | 156.5 | 13.8 | 228.6 | 29.3 | 22.9 | 42.2 | +46.1% | 3.15 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 28770 | 572.9 | 141.8 | 19.8 | 405.4 | 566.9 | -99.5% | 70.6 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | 46722 | 1965 | 2569 | 359.2 | 1412 | 1346 | -94.5% | 31.3 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 221.4 | 45.8 | 83.6 | 30.6 | 38.9 | 150.3 | -62.2% | 3.54 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q1_p99_us | 1131 | 361.6 | 1994 | 477.5 | 423.5 | 595.5 | +76.3% | 2.04 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 188.1 | 54.7 | 226.0 | 40.5 | 48.1 | 222.7 | +20.1% | 0.79 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q1_p50_us | 329.3 | 60.6 | 130.4 | 28.1 | 47.2 | 220.2 | -60.4% | 4.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p99_us | 1410 | 258.1 | 2470 | 235.0 | 246.8 | 1045 | +75.2% | 4.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 250.7 | 47.7 | 270.7 | 46.1 | 46.9 | 134.5 | +8.0% | 0.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p99_us | 2062 | 567.2 | 3736 | 1165 | 916.5 | 1611 | +81.2% | 1.83 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | write_p50_us | 533.8 | 28.8 | 1370 | 46.6 | 38.7 | 179.2 | +156.6% | 21.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | write_p99_us | 1418 | 502.8 | 5233 | 311.6 | 418.3 | 876.9 | +269.0% | 9.12 | **N better** |
| N vs H2 | commits_per_s | 1663 | 136.0 | 590.9 | 23.4 | 97.6 | 167.2 | -64.5% | 11.0 | **N better** |
| N vs H2 | pss_mib | 88.2 | 0.43 | 43.3 | 0.01 | 0.31 | 4.84 | -50.9% | 147.0 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 19475 | 4184 | -840.6 | 178.3 | 2961 | 107248 | -104.3% | 6.86 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 721.4 | 53.5 | 104.4 | 12.5 | 38.9 | 58.5 | -85.5% | 15.9 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p99_us | 2520 | 522.3 | 497.4 | 66.4 | 372.3 | 1374 | -80.3% | 5.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 578.2 | 32.0 | 228.6 | 29.3 | 30.7 | 83.2 | -60.5% | 11.4 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 306.9 | 37.8 | 276.3 | 38.3 | 38.0 | 158.2 | -10.0% | 0.81 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q4_p50_us | 12332 | 784.6 | 860.3 | 52.5 | 556.0 | 2196 | -93.0% | 20.6 | **H2 better** |
| P+ vs H2 | q5_p50_us | 42873 | 2784 | 141.8 | 19.8 | 1969 | 2323 | -99.7% | 21.7 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q6_p50_us | 41883 | 3008 | 2569 | 359.2 | 2142 | 7347 | -93.9% | 18.4 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 933.9 | 118.7 | 83.6 | 30.6 | 86.7 | 643.2 | -91.0% | 9.81 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q1_p99_us | 7591 | 1386 | 1994 | 477.5 | 1036 | 6003 | -73.7% | 5.40 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 747.9 | 84.7 | 226.0 | 40.5 | 66.4 | 740.3 | -69.8% | 7.86 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c4_q1_p50_us | 2008 | 88.0 | 130.4 | 28.1 | 65.3 | 312.5 | -93.5% | 28.7 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 10066 | 1854 | 2470 | 235.0 | 1321 | 934.7 | -75.5% | 5.75 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1838 | 238.2 | 270.7 | 46.1 | 171.6 | 46.5 | -85.3% | 9.13 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 7339 | 3018 | 3736 | 1165 | 2288 | 6153 | -49.1% | 1.57 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 962.0 | 53.8 | 1370 | 46.6 | 50.3 | 127.0 | +42.4% | 8.10 | **P+ better** |
| P+ vs H2 | write_p99_us | 3297 | 934.6 | 5233 | 311.6 | 696.6 | 1050 | +58.7% | 2.78 | no difference |
| P+ vs H2 | commits_per_s | 851.4 | 110.3 | 590.9 | 23.4 | 79.7 | 85.5 | -30.6% | 3.27 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.03 | 43.3 | 0.01 | 0.03 | 0.09 | +0.0% | 0.54 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | -786.9 | 171.6 | -840.6 | 178.3 | 175.0 | 2670 | +6.8% | 0.31 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 183.8 | 31.0 | 738.6 | 25.3 | 28.2 | 75.3 | +302.0% | 19.6 | **N better** |
| N vs P+ | q1_p99_us | 805.0 | 311.5 | 2543 | 713.3 | 550.4 | 544.1 | +215.9% | 3.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 147.1 | 10.8 | 601.6 | 41.0 | 29.9 | 63.5 | +308.9% | 15.2 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 28662 | 1447 | 39762 | 2130 | 1821 | 1021 | +38.7% | 6.10 | **N better** |
| N vs P+ | q6_p50_us | 44456 | 2124 | 41169 | 1188 | 1721 | 3116 | -7.4% | 1.91 | no difference |
| N vs P+ | c2_q1_p50_us | 256.1 | 22.0 | 902.5 | 86.5 | 63.1 | 232.9 | +252.4% | 10.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 1295 | 481.4 | 5683 | 775.1 | 645.2 | 1030 | +338.7% | 6.80 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 200.7 | 15.2 | 725.0 | 55.2 | 40.5 | 145.7 | +261.3% | 12.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 416.8 | 56.7 | 2164 | 254.2 | 184.2 | 203.8 | +419.2% | 9.49 | **N better** |
| N vs P+ | c4_q1_p99_us | 1883 | 356.1 | 9011 | 1515 | 1100 | 2654 | +378.6% | 6.48 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 328.9 | 59.1 | 1806 | 140.1 | 107.5 | 342.7 | +449.2% | 13.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1908 | 551.1 | 9104 | 1730 | 1284 | 862.5 | +377.1% | 5.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 489.8 | 49.6 | 941.9 | 64.8 | 57.7 | 75.4 | +92.3% | 7.83 | **N better** |
| N vs P+ | write_p99_us | 1578 | 642.8 | 3458 | 740.5 | 693.4 | 162.8 | +119.2% | 2.71 | no difference |
| N vs P+ | commits_per_s | 1798 | 68.6 | 863.4 | 69.8 | 69.2 | 632.2 | -52.0% | 13.5 | **N better** |
| N vs P+ | pss_mib | 87.5 | 1.08 | 43.5 | 0.03 | 0.76 | 2.83 | -50.3% | 57.8 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 20496 | 3394 | -744.9 | 147.2 | 2402 | 38818 | -103.6% | 8.84 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 183.8 | 31.0 | 97.1 | 10.1 | 23.0 | 23.8 | -47.2% | 3.77 | **P better** |
| N vs P | q1_p99_us | 805.0 | 311.5 | 731.8 | 271.2 | 292.0 | 442.8 | -9.1% | 0.25 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 147.1 | 10.8 | 209.6 | 17.5 | 14.5 | 53.4 | +42.4% | 4.30 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 28662 | 1447 | 137.3 | 24.6 | 1023 | 234.3 | -99.5% | 27.9 | **P better** |
| N vs P | q6_p50_us | 44456 | 2124 | 3725 | 611.3 | 1563 | 2229 | -91.6% | 26.1 | **P better** |
| N vs P | c2_q1_p50_us | 256.1 | 22.0 | 147.5 | 46.5 | 36.4 | 157.6 | -42.4% | 2.99 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 1295 | 481.4 | 1770 | 383.5 | 435.2 | 2152 | +36.6% | 1.09 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 200.7 | 15.2 | 271.1 | 109.6 | 78.3 | 106.4 | +35.1% | 0.90 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 416.8 | 56.7 | 128.9 | 15.8 | 41.6 | 130.0 | -69.1% | 6.92 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 1883 | 356.1 | 3193 | 781.5 | 607.3 | 554.0 | +69.6% | 2.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 328.9 | 59.1 | 277.6 | 33.3 | 48.0 | 185.6 | -15.6% | 1.07 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 1908 | 551.1 | 3491 | 657.3 | 606.6 | 1028 | +82.9% | 2.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 489.8 | 49.6 | 122183 | 3659 | 2588 | 174.5 | +24843.6% | 47.0 | **N better** |
| N vs P | write_p99_us | 1578 | 642.8 | 160534 | 12324 | 8726 | 9108 | +10075.5% | 18.2 | **N better** |
| N vs P | commits_per_s | 1798 | 68.6 | 8.03 | 0.31 | 48.5 | 598.9 | -99.6% | 36.9 | **N better** |
| N vs P | pss_mib | 87.5 | 1.08 | 43.4 | 0.02 | 0.76 | 2.78 | -50.4% | 57.8 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 20496 | 3394 | -723.7 | 168.8 | 2403 | 38665 | -103.5% | 8.83 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 738.6 | 25.3 | 97.1 | 10.1 | 19.2 | 72.6 | -86.9% | 33.4 | **P better** |
| P+ vs P | q1_p99_us | 2543 | 713.3 | 731.8 | 271.2 | 539.6 | 477.7 | -71.2% | 3.36 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 601.6 | 41.0 | 209.6 | 17.5 | 31.5 | 76.0 | -65.2% | 12.4 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 305.4 | 47.0 | 222.8 | 37.9 | 42.7 | 61.3 | -27.0% | 1.93 | no difference |
| P+ vs P | q4_p50_us | 13030 | 763.0 | 757.9 | 89.2 | 543.2 | 930.2 | -94.2% | 22.6 | **P better** |
| P+ vs P | q5_p50_us | 39762 | 2130 | 137.3 | 24.6 | 1507 | 994.3 | -99.7% | 26.3 | **P better** |
| P+ vs P | q6_p50_us | 41169 | 1188 | 3725 | 611.3 | 945.0 | 2291 | -91.0% | 39.6 | **P better** |
| P+ vs P | c2_q1_p50_us | 902.5 | 86.5 | 147.5 | 46.5 | 69.4 | 204.6 | -83.7% | 10.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5683 | 775.1 | 1770 | 383.5 | 611.5 | 2261 | -68.9% | 6.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 725.0 | 55.2 | 271.1 | 109.6 | 86.8 | 106.8 | -62.6% | 5.23 | **P better** |
| P+ vs P | c4_q1_p50_us | 2164 | 254.2 | 128.9 | 15.8 | 180.1 | 233.0 | -94.0% | 11.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 9011 | 1515 | 3193 | 781.5 | 1205 | 2612 | -64.6% | 4.83 | **P better** |
| P+ vs P | c4_q2_p50_us | 1806 | 140.1 | 277.6 | 33.3 | 101.8 | 342.5 | -84.6% | 15.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 9104 | 1730 | 3491 | 657.3 | 1308 | 833.7 | -61.7% | 4.29 | **P better** |
| P+ vs P | write_p50_us | 941.9 | 64.8 | 122183 | 3659 | 2588 | 157.7 | +12871.7% | 46.8 | **P+ better** |
| P+ vs P | write_p99_us | 3458 | 740.5 | 160534 | 12324 | 8730 | 9108 | +4542.1% | 18.0 | **P+ better** |
| P+ vs P | commits_per_s | 863.4 | 69.8 | 8.03 | 0.31 | 49.4 | 202.5 | -99.1% | 17.3 | **P+ better** |
| P+ vs P | pss_mib | 43.5 | 0.03 | 43.4 | 0.02 | 0.03 | 0.48 | -0.0% | 0.44 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -744.9 | 147.2 | -723.7 | 168.8 | 158.3 | 4114 | -2.8% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 97.1 | 10.1 | 96.4 | 5.97 | 8.28 | 9.19 | -0.7% | 0.09 | BELOW FLOOR |
| P vs M | q1_p99_us | 731.8 | 271.2 | 859.2 | 251.8 | 261.7 | 362.1 | +17.4% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 209.6 | 17.5 | 204.5 | 14.7 | 16.2 | 48.0 | -2.4% | 0.32 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 222.8 | 37.9 | 270.3 | 37.3 | 37.6 | 141.1 | +21.3% | 1.26 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 757.9 | 89.2 | 786.6 | 43.0 | 70.0 | 154.9 | +3.8% | 0.41 | BELOW FLOOR |
| P vs M | q5_p50_us | 137.3 | 24.6 | 152.5 | 16.2 | 20.8 | 35.8 | +11.1% | 0.73 | BELOW FLOOR |
| P vs M | q6_p50_us | 3725 | 611.3 | 3876 | 293.8 | 479.6 | 514.3 | +4.1% | 0.32 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 147.5 | 46.5 | 119.3 | 29.1 | 38.8 | 83.9 | -19.1% | 0.73 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q1_p99_us | 1770 | 383.5 | 2231 | 591.5 | 498.5 | 2346 | +26.1% | 0.93 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 271.1 | 109.6 | 269.0 | 65.6 | 90.3 | 72.6 | -0.7% | 0.02 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 128.9 | 15.8 | 135.9 | 11.5 | 13.8 | 123.4 | +5.4% | 0.51 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 3193 | 781.5 | 2367 | 421.6 | 627.9 | 768.2 | -25.9% | 1.32 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 277.6 | 33.3 | 305.5 | 35.4 | 34.4 | 161.5 | +10.0% | 0.81 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p99_us | 3491 | 657.3 | 3317 | 336.7 | 522.2 | 3040 | -5.0% | 0.33 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 122183 | 3659 | 124316 | 2048 | 2965 | 177.4 | +1.7% | 0.72 | no difference |
| P vs M | write_p99_us | 160534 | 12324 | 161285 | 10264 | 11341 | 24423 | +0.5% | 0.07 | BELOW FLOOR |
| P vs M | commits_per_s | 8.03 | 0.31 | 8.01 | 0.16 | 0.25 | 0.18 | -0.2% | 0.07 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.02 | 43.4 | 0.01 | 0.02 | 0.04 | -0.2% | 3.50 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -723.7 | 168.8 | -728.0 | 141.0 | 155.5 | 2381 | +0.6% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 183.8 | 31.0 | 384.6 | 50.1 | 41.6 | 22.8 | +109.3% | 4.83 | **N better** |
| N vs H3 | q1_p99_us | 805.0 | 311.5 | 1417 | 413.9 | 366.3 | 1378 | +76.0% | 1.67 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 147.1 | 10.8 | 383.3 | 37.6 | 27.7 | 57.8 | +160.5% | 8.54 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 28662 | 1447 | 39950 | 3503 | 2680 | 420.7 | +39.4% | 4.21 | **N better** |
| N vs H3 | q6_p50_us | 44456 | 2124 | 42946 | 3121 | 2669 | 6989 | -3.4% | 0.57 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 256.1 | 22.0 | 435.9 | 63.7 | 47.7 | 227.0 | +70.2% | 3.77 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q1_p99_us | 1295 | 481.4 | 2279 | 226.0 | 376.0 | 631.0 | +75.9% | 2.62 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 200.7 | 15.2 | 406.5 | 49.2 | 36.4 | 228.0 | +102.6% | 5.65 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p50_us | 416.8 | 56.7 | 598.6 | 55.3 | 56.0 | 174.0 | +43.6% | 3.25 | **N better** |
| N vs H3 | c4_q1_p99_us | 1883 | 356.1 | 2844 | 975.0 | 734.0 | 537.8 | +51.0% | 1.31 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 328.9 | 59.1 | 680.9 | 116.5 | 92.4 | 212.4 | +107.0% | 3.81 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1908 | 551.1 | 3019 | 1076 | 854.9 | 1170 | +58.2% | 1.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 489.8 | 49.6 | 1240 | 37.7 | 44.1 | 75.2 | +153.0% | 17.0 | **N better** |
| N vs H3 | write_p99_us | 1578 | 642.8 | 5456 | 1661 | 1259 | 430.2 | +245.8% | 3.08 | **N better** |
| N vs H3 | commits_per_s | 1798 | 68.6 | 598.0 | 54.3 | 61.9 | 600.9 | -66.8% | 19.4 | **N better** |
| N vs H3 | pss_mib | 87.5 | 1.08 | 106.1 | 0.11 | 0.77 | 2.81 | +21.3% | 24.3 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 20496 | 3394 | 19699 | 3894 | 3653 | 54705 | -3.9% | 0.22 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 738.6 | 25.3 | 384.6 | 50.1 | 39.6 | 72.3 | -47.9% | 8.93 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2543 | 713.3 | 1417 | 413.9 | 583.1 | 1390 | -44.3% | 1.93 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 601.6 | 41.0 | 383.3 | 37.6 | 39.3 | 79.1 | -36.3% | 5.55 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 305.4 | 47.0 | 337.3 | 47.8 | 47.4 | 91.0 | +10.4% | 0.67 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 13030 | 763.0 | 12512 | 518.4 | 652.3 | 990.9 | -4.0% | 0.79 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 39762 | 2130 | 39950 | 3503 | 2899 | 1054 | +0.5% | 0.06 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 41169 | 1188 | 42946 | 3121 | 2362 | 7009 | +4.3% | 0.75 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 902.5 | 86.5 | 435.9 | 63.7 | 75.9 | 261.8 | -51.7% | 6.14 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q1_p99_us | 5683 | 775.1 | 2279 | 226.0 | 570.9 | 936.6 | -59.9% | 5.96 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 725.0 | 55.2 | 406.5 | 49.2 | 52.3 | 228.1 | -43.9% | 6.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 2164 | 254.2 | 598.6 | 55.3 | 183.9 | 260.1 | -72.3% | 8.51 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 9011 | 1515 | 2844 | 975.0 | 1274 | 2609 | -68.4% | 4.84 | **H3 better** |
| P+ vs H3 | c4_q2_p50_us | 1806 | 140.1 | 680.9 | 116.5 | 128.8 | 357.8 | -62.3% | 8.74 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 9104 | 1730 | 3019 | 1076 | 1440 | 1004 | -66.8% | 4.22 | **H3 better** |
| P+ vs H3 | write_p50_us | 941.9 | 64.8 | 1240 | 37.7 | 53.0 | 7.06 | +31.6% | 5.62 | **P+ better** |
| P+ vs H3 | write_p99_us | 3458 | 740.5 | 5456 | 1661 | 1286 | 418.3 | +57.8% | 1.55 | no difference |
| P+ vs H3 | commits_per_s | 863.4 | 69.8 | 598.0 | 54.3 | 62.5 | 208.6 | -30.7% | 4.25 | **P+ better** |
| P+ vs H3 | pss_mib | 43.5 | 0.03 | 106.1 | 0.11 | 0.08 | 0.59 | +144.2% | 795.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -744.9 | 147.2 | 19699 | 3894 | 2756 | 38917 | -2744.7% | 7.42 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 183.8 | 31.0 | 1359 | 63.4 | 49.9 | 123.8 | +639.5% | 23.6 | **N better** |
| N vs T | q1_p99_us | 805.0 | 311.5 | 4125 | 844.2 | 636.2 | 5187 | +412.5% | 5.22 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | q2_p50_us | 147.1 | 10.8 | 1447 | 57.4 | 41.3 | 38.0 | +883.7% | 31.5 | **N better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 256.1 | 22.0 | 1781 | 76.7 | 56.5 | 300.4 | +595.3% | 27.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q1_p99_us | 1295 | 481.4 | 5276 | 1588 | 1174 | 1770 | +307.3% | 3.39 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | c2_q2_p50_us | 200.7 | 15.2 | 1831 | 130.5 | 92.9 | 468.0 | +812.3% | 17.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p50_us | 416.8 | 56.7 | 2575 | 278.1 | 200.7 | 317.9 | +517.9% | 10.8 | **N better** |
| N vs T | c4_q1_p99_us | 1883 | 356.1 | 6835 | 2712 | 1934 | 1262 | +263.0% | 2.56 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p50_us | 328.9 | 59.1 | 2539 | 272.9 | 197.4 | 343.6 | +671.9% | 11.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p99_us | 1908 | 551.1 | 5812 | 2043 | 1496 | 1814 | +204.5% | 2.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | write_p50_us | 489.8 | 49.6 | 1157 | 90.3 | 72.8 | 103.4 | +136.2% | 9.16 | **N better** |
| N vs T | write_p99_us | 1578 | 642.8 | 1858 | 221.2 | 480.7 | 354.6 | +17.8% | 0.58 | BELOW FLOOR |
| N vs T | commits_per_s | 1798 | 68.6 | 752.2 | 83.5 | 76.4 | 677.3 | -58.2% | 13.7 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | pss_mib | 87.5 | 1.08 | 2581 | 0.14 | 0.77 | 2.79 | +2849.7% | 3242 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 20496 | 3394 | -356.6 | 113.6 | 2401 | 38651 | -101.7% | 8.68 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 738.6 | 25.3 | 1359 | 63.4 | 48.3 | 141.5 | +84.0% | 12.9 | **P+ better** |
| P+ vs T | q1_p99_us | 2543 | 713.3 | 4125 | 844.2 | 781.5 | 5190 | +62.2% | 2.02 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | q2_p50_us | 601.6 | 41.0 | 1447 | 57.4 | 49.8 | 66.0 | +140.6% | 17.0 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 902.5 | 86.5 | 1781 | 76.7 | 81.7 | 327.5 | +97.3% | 10.7 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 5683 | 775.1 | 5276 | 1588 | 1250 | 1901 | -7.2% | 0.33 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c2_q2_p50_us | 725.0 | 55.2 | 1831 | 130.5 | 100.2 | 468.1 | +152.5% | 11.0 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2164 | 254.2 | 2575 | 278.1 | 266.4 | 372.1 | +19.0% | 1.54 | no difference |
| P+ vs T | c4_q1_p99_us | 9011 | 1515 | 6835 | 2712 | 2197 | 2848 | -24.1% | 0.99 | BELOW FLOOR |
| P+ vs T | c4_q2_p50_us | 1806 | 140.1 | 2539 | 272.9 | 216.9 | 448.2 | +40.6% | 3.38 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 9104 | 1730 | 5812 | 2043 | 1893 | 1711 | -36.2% | 1.74 | no difference |
| P+ vs T | write_p50_us | 941.9 | 64.8 | 1157 | 90.3 | 78.6 | 71.4 | +22.8% | 2.73 | no difference |
| P+ vs T | write_p99_us | 3458 | 740.5 | 1858 | 221.2 | 546.5 | 340.0 | -46.3% | 2.93 | no difference |
| P+ vs T | commits_per_s | 863.4 | 69.8 | 752.2 | 83.5 | 76.9 | 375.6 | -12.9% | 1.45 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | pss_mib | 43.5 | 0.03 | 2581 | 0.14 | 0.10 | 0.51 | +5839.9% | 24444 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -744.9 | 147.2 | -356.6 | 113.6 | 131.5 | 3986 | -52.1% | 2.95 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 384.6 | 50.1 | 1359 | 63.4 | 57.1 | 121.9 | +253.3% | 17.1 | **H3 better** |
| H3 vs T | q1_p99_us | 1417 | 413.9 | 4125 | 844.2 | 664.8 | 5343 | +191.2% | 4.07 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | q2_p50_us | 383.3 | 37.6 | 1447 | 57.4 | 48.5 | 60.6 | +277.6% | 21.9 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 435.9 | 63.7 | 1781 | 76.7 | 70.5 | 323.3 | +308.6% | 19.1 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c2_q1_p99_us | 2279 | 226.0 | 5276 | 1588 | 1134 | 1718 | +131.5% | 2.64 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | c2_q2_p50_us | 406.5 | 49.2 | 1831 | 130.5 | 98.6 | 499.8 | +350.4% | 14.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p50_us | 598.6 | 55.3 | 2575 | 278.1 | 200.5 | 356.7 | +330.2% | 9.86 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 2844 | 975.0 | 6835 | 2712 | 2038 | 1166 | +140.4% | 1.96 | no difference |
| H3 vs T | c4_q2_p50_us | 680.9 | 116.5 | 2539 | 272.9 | 209.8 | 358.6 | +272.9% | 8.85 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 3019 | 1076 | 5812 | 2043 | 1633 | 1885 | +92.5% | 1.71 | no difference |
| H3 vs T | write_p50_us | 1240 | 37.7 | 1157 | 90.3 | 69.2 | 71.1 | -6.7% | 1.20 | no difference |
| H3 vs T | write_p99_us | 5456 | 1661 | 1858 | 221.2 | 1185 | 523.6 | -66.0% | 3.04 | **T better** |
| H3 vs T | commits_per_s | 598.0 | 54.3 | 752.2 | 83.5 | 70.4 | 320.2 | +25.8% | 2.19 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | pss_mib | 106.1 | 0.11 | 2581 | 0.14 | 0.13 | 0.38 | +2332.3% | 19375 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 19699 | 3894 | -356.6 | 113.6 | 2755 | 38752 | -101.8% | 7.28 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 183.8 | 31.0 | 937.6 | 38.1 | 34.7 | 25.1 | +410.2% | 21.7 | **N better** |
| N vs H1 | q1_p99_us | 805.0 | 311.5 | 4265 | 535.6 | 438.1 | 806.0 | +429.7% | 7.90 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p50_us | 147.1 | 10.8 | 781.0 | 47.9 | 34.7 | 29.6 | +430.8% | 18.2 | **N better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 28662 | 1447 | 219803 | 219367 | 155119 | 21338 | +666.9% | 1.23 | no difference |
| N vs H1 | q6_p50_us | 44456 | 2124 | 6808 | 681.7 | 1577 | 2171 | -84.7% | 23.9 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 256.1 | 22.0 | 1008 | 65.0 | 48.5 | 145.7 | +293.5% | 15.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q1_p99_us | 1295 | 481.4 | 5740 | 625.7 | 558.2 | 2492 | +343.1% | 7.96 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | c2_q2_p50_us | 200.7 | 15.2 | 906.4 | 39.5 | 30.0 | 215.0 | +351.7% | 23.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p50_us | 416.8 | 56.7 | 1005 | 63.0 | 59.9 | 71.6 | +141.0% | 9.81 | **N better** |
| N vs H1 | c4_q1_p99_us | 1883 | 356.1 | 4158 | 835.5 | 642.2 | 2576 | +120.8% | 3.54 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | c4_q2_p50_us | 328.9 | 59.1 | 866.9 | 23.1 | 44.9 | 183.1 | +163.6% | 12.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p99_us | 1908 | 551.1 | 4828 | 1578 | 1182 | 3003 | +153.0% | 2.47 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| N vs H1 | write_p50_us | 489.8 | 49.6 | 1583 | 80.4 | 66.8 | 80.9 | +223.1% | 16.3 | **N better** |
| N vs H1 | write_p99_us | 1578 | 642.8 | 5196 | 311.7 | 505.1 | 138.7 | +229.3% | 7.16 | **N better** |
| N vs H1 | commits_per_s | 1798 | 68.6 | 530.2 | 42.8 | 57.2 | 601.2 | -70.5% | 22.2 | **N better** |
| N vs H1 | pss_mib | 87.5 | 1.08 | 432.5 | 6.93 | 4.96 | 2.79 | +394.2% | 69.5 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 20496 | 3394 | 13596 | 2144 | 2839 | 64030 | -33.7% | 2.43 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| P+ vs H1 | q1_p50_us | 738.6 | 25.3 | 937.6 | 38.1 | 32.3 | 73.0 | +26.9% | 6.16 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2543 | 713.3 | 4265 | 535.6 | 630.8 | 825.7 | +67.7% | 2.73 | no difference |
| P+ vs H1 | q2_p50_us | 601.6 | 41.0 | 781.0 | 47.9 | 44.6 | 61.6 | +29.8% | 4.02 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 305.4 | 47.0 | 915.9 | 80.1 | 65.7 | 71.8 | +199.9% | 9.30 | **P+ better** |
| P+ vs H1 | q4_p50_us | 13030 | 763.0 | 25549 | 2602 | 1917 | 3423 | +96.1% | 6.53 | **P+ better** |
| P+ vs H1 | q5_p50_us | 39762 | 2130 | 219803 | 219367 | 155123 | 21360 | +452.8% | 1.16 | no difference |
| P+ vs H1 | q6_p50_us | 41169 | 1188 | 6808 | 681.7 | 968.8 | 2235 | -83.5% | 35.5 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 902.5 | 86.5 | 1008 | 65.0 | 76.5 | 195.6 | +11.7% | 1.38 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 5683 | 775.1 | 5740 | 625.7 | 704.4 | 2587 | +1.0% | 0.08 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | c2_q2_p50_us | 725.0 | 55.2 | 906.4 | 39.5 | 48.0 | 215.2 | +25.0% | 3.78 | BELOW FLOOR |
| P+ vs H1 | c4_q1_p50_us | 2164 | 254.2 | 1005 | 63.0 | 185.2 | 206.2 | -53.6% | 6.26 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 9011 | 1515 | 4158 | 835.5 | 1223 | 3626 | -53.9% | 3.97 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | c4_q2_p50_us | 1806 | 140.1 | 866.9 | 23.1 | 100.4 | 341.2 | -52.0% | 9.36 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 9104 | 1730 | 4828 | 1578 | 1656 | 2942 | -47.0% | 2.58 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | write_p50_us | 941.9 | 64.8 | 1583 | 80.4 | 73.0 | 30.6 | +68.0% | 8.77 | **P+ better** |
| P+ vs H1 | write_p99_us | 3458 | 740.5 | 5196 | 311.7 | 568.1 | 95.4 | +50.2% | 3.06 | **P+ better** |
| P+ vs H1 | commits_per_s | 863.4 | 69.8 | 530.2 | 42.8 | 57.9 | 209.3 | -38.6% | 5.76 | **P+ better** |
| P+ vs H1 | pss_mib | 43.5 | 0.03 | 432.5 | 6.93 | 4.90 | 0.49 | +895.3% | 79.4 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -744.9 | 147.2 | 13596 | 2144 | 1520 | 51203 | -1925.3% | 9.44 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 183.8 | 31.0 | 100.2 | 17.4 | 25.1 | 56.9 | -45.4% | 3.33 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q1_p99_us | 805.0 | 311.5 | 722.8 | 243.2 | 279.4 | 365.2 | -10.2% | 0.29 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 147.1 | 10.8 | 195.0 | 17.5 | 14.5 | 152.8 | +32.5% | 3.29 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 28662 | 1447 | 166.2 | 24.6 | 1023 | 241.4 | -99.4% | 27.8 | **H2 better** |
| N vs H2 | q6_p50_us | 44456 | 2124 | 3099 | 347.4 | 1522 | 2172 | -93.0% | 27.2 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 256.1 | 22.0 | 88.4 | 23.3 | 22.7 | 138.7 | -65.5% | 7.40 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q1_p99_us | 1295 | 481.4 | 2266 | 525.2 | 503.7 | 584.6 | +74.9% | 1.93 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q2_p50_us | 200.7 | 15.2 | 208.4 | 48.8 | 36.2 | 102.8 | +3.8% | 0.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p50_us | 416.8 | 56.7 | 110.8 | 24.1 | 43.5 | 63.7 | -73.4% | 7.03 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p99_us | 1883 | 356.1 | 2363 | 375.3 | 365.8 | 587.9 | +25.5% | 1.31 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 328.9 | 59.1 | 280.3 | 52.8 | 56.0 | 139.3 | -14.8% | 0.87 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p99_us | 1908 | 551.1 | 3503 | 827.6 | 703.1 | 4691 | +83.5% | 2.27 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | write_p50_us | 489.8 | 49.6 | 1469 | 142.0 | 106.4 | 319.3 | +199.9% | 9.20 | **N better** |
| N vs H2 | write_p99_us | 1578 | 642.8 | 5798 | 771.7 | 710.2 | 212.5 | +267.5% | 5.94 | **N better** |
| N vs H2 | commits_per_s | 1798 | 68.6 | 585.0 | 55.3 | 62.3 | 618.0 | -67.5% | 19.5 | **N better** |
| N vs H2 | pss_mib | 87.5 | 1.08 | 43.4 | 0.04 | 0.76 | 2.78 | -50.4% | 57.8 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 20496 | 3394 | -823.7 | 174.2 | 2403 | 38674 | -104.0% | 8.87 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 738.6 | 25.3 | 100.2 | 17.4 | 21.7 | 89.1 | -86.4% | 29.4 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q1_p99_us | 2543 | 713.3 | 722.8 | 243.2 | 532.9 | 406.7 | -71.6% | 3.42 | **H2 better** |
| P+ vs H2 | q2_p50_us | 601.6 | 41.0 | 195.0 | 17.5 | 31.5 | 162.1 | -67.6% | 12.9 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 305.4 | 47.0 | 268.2 | 46.1 | 46.6 | 115.8 | -12.2% | 0.80 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q4_p50_us | 13030 | 763.0 | 880.3 | 110.5 | 545.1 | 924.9 | -93.2% | 22.3 | **H2 better** |
| P+ vs H2 | q5_p50_us | 39762 | 2130 | 166.2 | 24.6 | 1507 | 996.0 | -99.6% | 26.3 | **H2 better** |
| P+ vs H2 | q6_p50_us | 41169 | 1188 | 3099 | 347.4 | 875.5 | 2235 | -92.5% | 43.5 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 902.5 | 86.5 | 88.4 | 23.3 | 63.3 | 190.4 | -90.2% | 12.9 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5683 | 775.1 | 2266 | 525.2 | 662.0 | 906.0 | -60.1% | 5.16 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 725.0 | 55.2 | 208.4 | 48.8 | 52.1 | 103.2 | -71.3% | 9.91 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 2164 | 254.2 | 110.8 | 24.1 | 180.6 | 203.6 | -94.9% | 11.4 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 9011 | 1515 | 2363 | 375.3 | 1104 | 2620 | -73.8% | 6.02 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1806 | 140.1 | 280.3 | 52.8 | 105.9 | 319.8 | -84.5% | 14.4 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 9104 | 1730 | 3503 | 827.6 | 1356 | 4652 | -61.5% | 4.13 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | write_p50_us | 941.9 | 64.8 | 1469 | 142.0 | 110.4 | 310.4 | +56.0% | 4.77 | **P+ better** |
| P+ vs H2 | write_p99_us | 3458 | 740.5 | 5798 | 771.7 | 756.3 | 187.1 | +67.7% | 3.09 | **P+ better** |
| P+ vs H2 | commits_per_s | 863.4 | 69.8 | 585.0 | 55.3 | 63.0 | 253.6 | -32.2% | 4.42 | **P+ better** |
| P+ vs H2 | pss_mib | 43.5 | 0.03 | 43.4 | 0.04 | 0.03 | 0.48 | -0.1% | 1.15 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | -744.9 | 147.2 | -823.7 | 174.2 | 161.2 | 4202 | +10.6% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 226.7 | 37.6 | 785.9 | 12.3 | 28.0 | 68.8 | +246.6% | 20.0 | **N better** |
| N vs P+ | q1_p99_us | 891.1 | 179.0 | 2740 | 777.7 | 564.3 | 1982 | +207.5% | 3.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 169.4 | 11.2 | 618.8 | 13.6 | 12.5 | 173.7 | +265.3% | 36.0 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 30423 | 1216 | 40758 | 1901 | 1596 | 4374 | +34.0% | 6.48 | **N better** |
| N vs P+ | q6_p50_us | 47610 | 3020 | 40997 | 3698 | 3376 | 20722 | -13.9% | 1.96 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q1_p50_us | 244.6 | 45.3 | 963.1 | 142.2 | 105.5 | 45.7 | +293.7% | 6.81 | **N better** |
| N vs P+ | c2_q1_p99_us | 1394 | 300.6 | 7170 | 1437 | 1038 | 987.5 | +414.5% | 5.56 | **N better** |
| N vs P+ | c2_q2_p50_us | 166.9 | 35.3 | 765.9 | 83.5 | 64.1 | 316.4 | +359.0% | 9.34 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 359.7 | 31.3 | 1984 | 69.8 | 54.1 | 200.0 | +451.4% | 30.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1848 | 372.5 | 10161 | 987.5 | 746.3 | 1144 | +449.8% | 11.1 | **N better** |
| N vs P+ | c4_q2_p50_us | 263.6 | 12.9 | 1765 | 142.1 | 100.9 | 246.5 | +569.4% | 14.9 | **N better** |
| N vs P+ | c4_q2_p99_us | 2073 | 643.6 | 8286 | 2752 | 1998 | 4063 | +299.8% | 3.11 | **N better** |
| N vs P+ | write_p50_us | 547.3 | 45.3 | 986.0 | 55.5 | 50.7 | 86.1 | +80.2% | 8.66 | **N better** |
| N vs P+ | write_p99_us | 1612 | 504.9 | 3593 | 468.8 | 487.2 | 2030 | +122.9% | 4.07 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 1553 | 185.8 | 833.5 | 60.4 | 138.1 | 159.3 | -46.3% | 5.21 | **N better** |
| N vs P+ | pss_mib | 86.9 | 2.38 | 43.3 | 0.02 | 1.69 | 2.61 | -50.2% | 25.9 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 20852 | 3507 | -827.0 | 169.2 | 2483 | 30506 | -104.0% | 8.73 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 226.7 | 37.6 | 82.7 | 15.8 | 28.8 | 34.1 | -63.5% | 5.00 | **P better** |
| N vs P | q1_p99_us | 891.1 | 179.0 | 618.3 | 118.5 | 151.8 | 62.2 | -30.6% | 1.80 | no difference |
| N vs P | q2_p50_us | 169.4 | 11.2 | 164.3 | 19.9 | 16.1 | 45.9 | -3.0% | 0.31 | BELOW FLOOR |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 30423 | 1216 | 137.0 | 20.5 | 859.6 | 4069 | -99.5% | 35.2 | **P better** |
| N vs P | q6_p50_us | 47610 | 3020 | 3729 | 501.6 | 2164 | 9458 | -92.2% | 20.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p50_us | 244.6 | 45.3 | 72.8 | 25.7 | 36.8 | 10.8 | -70.2% | 4.67 | **P better** |
| N vs P | c2_q1_p99_us | 1394 | 300.6 | 1325 | 297.8 | 299.2 | 36.1 | -4.9% | 0.23 | no difference |
| N vs P | c2_q2_p50_us | 166.9 | 35.3 | 193.9 | 29.1 | 32.4 | 189.9 | +16.2% | 0.83 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 359.7 | 31.3 | 89.7 | 11.4 | 23.5 | 165.4 | -75.1% | 11.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 1848 | 372.5 | 2633 | 196.1 | 297.7 | 1034 | +42.5% | 2.64 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 263.6 | 12.9 | 214.8 | 34.7 | 26.2 | 72.8 | -18.5% | 1.86 | BELOW FLOOR |
| N vs P | c4_q2_p99_us | 2073 | 643.6 | 5450 | 1953 | 1454 | 1124 | +162.9% | 2.32 | no difference |
| N vs P | write_p50_us | 547.3 | 45.3 | 118391 | 4413 | 3121 | 6048 | +21533.8% | 37.8 | **N better** |
| N vs P | write_p99_us | 1612 | 504.9 | 164461 | 11278 | 7983 | 5652 | +10102.0% | 20.4 | **N better** |
| N vs P | commits_per_s | 1553 | 185.8 | 8.04 | 0.23 | 131.3 | 155.7 | -99.5% | 11.8 | **N better** |
| N vs P | pss_mib | 86.9 | 2.38 | 43.4 | 0.04 | 1.69 | 2.62 | -50.1% | 25.8 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 20852 | 3507 | -747.7 | 151.7 | 2482 | 30769 | -103.6% | 8.70 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 785.9 | 12.3 | 82.7 | 15.8 | 14.1 | 68.2 | -89.5% | 49.8 | **P better** |
| P+ vs P | q1_p99_us | 2740 | 777.7 | 618.3 | 118.5 | 556.3 | 1983 | -77.4% | 3.81 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 618.8 | 13.6 | 164.3 | 19.9 | 17.0 | 177.7 | -73.4% | 26.7 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 360.7 | 36.5 | 233.3 | 46.0 | 41.5 | 64.7 | -35.3% | 3.07 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 13617 | 452.5 | 759.8 | 69.3 | 323.7 | 2449 | -94.4% | 39.7 | **P better** |
| P+ vs P | q5_p50_us | 40758 | 1901 | 137.0 | 20.5 | 1344 | 1606 | -99.7% | 30.2 | **P better** |
| P+ vs P | q6_p50_us | 40997 | 3698 | 3729 | 501.6 | 2638 | 18530 | -90.9% | 14.1 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q1_p50_us | 963.1 | 142.2 | 72.8 | 25.7 | 102.1 | 44.5 | -92.4% | 8.72 | **P better** |
| P+ vs P | c2_q1_p99_us | 7170 | 1437 | 1325 | 297.8 | 1038 | 988.0 | -81.5% | 5.63 | **P better** |
| P+ vs P | c2_q2_p50_us | 765.9 | 83.5 | 193.9 | 29.1 | 62.6 | 368.5 | -74.7% | 9.14 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p50_us | 1984 | 69.8 | 89.7 | 11.4 | 50.0 | 113.1 | -95.5% | 37.9 | **P better** |
| P+ vs P | c4_q1_p99_us | 10161 | 987.5 | 2633 | 196.1 | 711.9 | 1541 | -74.1% | 10.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1765 | 142.1 | 214.8 | 34.7 | 103.4 | 245.3 | -87.8% | 15.0 | **P better** |
| P+ vs P | c4_q2_p99_us | 8286 | 2752 | 5450 | 1953 | 2386 | 4216 | -34.2% | 1.19 | BELOW FLOOR |
| P+ vs P | write_p50_us | 986.0 | 55.5 | 118391 | 4413 | 3121 | 6049 | +11907.3% | 37.6 | **P+ better** |
| P+ vs P | write_p99_us | 3593 | 468.8 | 164461 | 11278 | 7982 | 6001 | +4477.8% | 20.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 833.5 | 60.4 | 8.04 | 0.23 | 42.7 | 33.4 | -99.0% | 19.3 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.02 | 43.4 | 0.04 | 0.03 | 0.22 | +0.3% | 3.76 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -827.0 | 169.2 | -747.7 | 151.7 | 160.7 | 4812 | -9.6% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 82.7 | 15.8 | 90.7 | 9.76 | 13.1 | 27.0 | +9.7% | 0.61 | BELOW FLOOR |
| P vs M | q1_p99_us | 618.3 | 118.5 | 677.2 | 154.5 | 137.7 | 63.1 | +9.5% | 0.43 | BELOW FLOOR |
| P vs M | q2_p50_us | 164.3 | 19.9 | 190.8 | 15.5 | 17.8 | 47.1 | +16.1% | 1.48 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 233.3 | 46.0 | 268.3 | 38.5 | 42.4 | 78.1 | +15.0% | 0.82 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q4_p50_us | 759.8 | 69.3 | 744.6 | 93.4 | 82.3 | 359.5 | -2.0% | 0.18 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q5_p50_us | 137.0 | 20.5 | 147.4 | 19.6 | 20.0 | 91.8 | +7.6% | 0.52 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | 3729 | 501.6 | 3311 | 567.3 | 535.5 | 1308 | -11.2% | 0.78 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p50_us | 72.8 | 25.7 | 69.2 | 36.7 | 31.7 | 113.6 | -4.9% | 0.11 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1325 | 297.8 | 2398 | 587.7 | 465.8 | 1209 | +80.9% | 2.30 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 193.9 | 29.1 | 227.2 | 75.6 | 57.3 | 190.1 | +17.2% | 0.58 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 89.7 | 11.4 | 120.0 | 24.9 | 19.4 | 84.6 | +33.7% | 1.56 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2633 | 196.1 | 2892 | 425.2 | 331.1 | 1038 | +9.8% | 0.78 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 214.8 | 34.7 | 346.0 | 64.8 | 52.0 | 127.9 | +61.1% | 2.53 | no difference |
| P vs M | c4_q2_p99_us | 5450 | 1953 | 3792 | 350.5 | 1403 | 1645 | -30.4% | 1.18 | no difference |
| P vs M | write_p50_us | 118391 | 4413 | 120184 | 7308 | 6037 | 11527 | +1.5% | 0.30 | BELOW FLOOR |
| P vs M | write_p99_us | 164461 | 11278 | 169114 | 17429 | 14679 | 33539 | +2.8% | 0.32 | BELOW FLOOR |
| P vs M | commits_per_s | 8.04 | 0.23 | 8.03 | 0.27 | 0.25 | 1.06 | -0.2% | 0.06 | BELOW FLOOR |
| P vs M | pss_mib | 43.4 | 0.04 | 43.4 | 0.02 | 0.03 | 0.25 | -0.1% | 0.85 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -747.7 | 151.7 | -794.8 | 132.8 | 142.6 | 4435 | +6.3% | 0.33 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 226.7 | 37.6 | 459.7 | 36.8 | 37.2 | 58.2 | +102.8% | 6.26 | **N better** |
| N vs H3 | q1_p99_us | 891.1 | 179.0 | 1408 | 163.4 | 171.4 | 1155 | +58.1% | 3.02 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p50_us | 169.4 | 11.2 | 467.8 | 51.8 | 37.5 | 116.2 | +176.1% | 7.96 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 30423 | 1216 | 39797 | 1372 | 1296 | 5511 | +30.8% | 7.23 | **N better** |
| N vs H3 | q6_p50_us | 47610 | 3020 | 47722 | 3671 | 3361 | 10578 | +0.2% | 0.03 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 244.6 | 45.3 | 517.0 | 57.7 | 51.9 | 44.4 | +111.4% | 5.25 | **N better** |
| N vs H3 | c2_q1_p99_us | 1394 | 300.6 | 1906 | 347.8 | 325.0 | 7278 | +36.7% | 1.58 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 166.9 | 35.3 | 497.8 | 73.6 | 57.7 | 105.9 | +198.3% | 5.74 | **N better** |
| N vs H3 | c4_q1_p50_us | 359.7 | 31.3 | 711.7 | 42.9 | 37.6 | 366.7 | +97.8% | 9.37 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 1848 | 372.5 | 3311 | 594.4 | 496.0 | 6519 | +79.1% | 2.95 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 263.6 | 12.9 | 684.9 | 52.5 | 38.2 | 173.5 | +159.8% | 11.0 | **N better** |
| N vs H3 | c4_q2_p99_us | 2073 | 643.6 | 3796 | 693.8 | 669.2 | 697.9 | +83.1% | 2.57 | no difference |
| N vs H3 | write_p50_us | 547.3 | 45.3 | 1471 | 78.1 | 63.8 | 22.8 | +168.8% | 14.5 | **N better** |
| N vs H3 | write_p99_us | 1612 | 504.9 | 4956 | 828.8 | 686.2 | 349.4 | +207.4% | 4.87 | **N better** |
| N vs H3 | commits_per_s | 1553 | 185.8 | 563.4 | 40.5 | 134.4 | 187.8 | -63.7% | 7.36 | **N better** |
| N vs H3 | pss_mib | 86.9 | 2.38 | 105.9 | 0.41 | 1.71 | 2.69 | +21.8% | 11.1 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 20852 | 3507 | 19851 | 3930 | 3725 | 48965 | -4.8% | 0.27 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 785.9 | 12.3 | 459.7 | 36.8 | 27.5 | 82.9 | -41.5% | 11.9 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2740 | 777.7 | 1408 | 163.4 | 561.9 | 2294 | -48.6% | 2.37 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 618.8 | 13.6 | 467.8 | 51.8 | 37.9 | 207.3 | -24.4% | 3.99 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 360.7 | 36.5 | 398.7 | 57.9 | 48.4 | 133.7 | +10.5% | 0.78 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 13617 | 452.5 | 14243 | 937.5 | 736.1 | 2566 | +4.6% | 0.85 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 40758 | 1901 | 39797 | 1372 | 1658 | 4048 | -2.4% | 0.58 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 40997 | 3698 | 47722 | 3671 | 3684 | 19126 | +16.4% | 1.83 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q1_p50_us | 963.1 | 142.2 | 517.0 | 57.7 | 108.5 | 61.9 | -46.3% | 4.11 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 7170 | 1437 | 1906 | 347.8 | 1046 | 7345 | -73.4% | 5.03 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 765.9 | 83.5 | 497.8 | 73.6 | 78.7 | 333.1 | -35.0% | 3.41 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p50_us | 1984 | 69.8 | 711.7 | 42.9 | 57.9 | 346.3 | -64.1% | 21.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 10161 | 987.5 | 3311 | 594.4 | 815.0 | 6618 | -67.4% | 8.41 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1765 | 142.1 | 684.9 | 52.5 | 107.1 | 291.5 | -61.2% | 10.1 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8286 | 2752 | 3796 | 693.8 | 2007 | 4122 | -54.2% | 2.24 | no difference |
| P+ vs H3 | write_p50_us | 986.0 | 55.5 | 1471 | 78.1 | 67.7 | 83.3 | +49.2% | 7.16 | **P+ better** |
| P+ vs H3 | write_p99_us | 3593 | 468.8 | 4956 | 828.8 | 673.3 | 2047 | +37.9% | 2.02 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 833.5 | 60.4 | 563.4 | 40.5 | 51.4 | 110.2 | -32.4% | 5.25 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.02 | 105.9 | 0.41 | 0.29 | 0.64 | +144.6% | 214.3 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -827.0 | 169.2 | 19851 | 3930 | 2782 | 38392 | -2500.5% | 7.43 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 226.7 | 37.6 | 1386 | 50.5 | 44.5 | 153.5 | +511.4% | 26.1 | **N better** |
| N vs T | q1_p99_us | 891.1 | 179.0 | 3879 | 1078 | 772.9 | 669.7 | +335.3% | 3.87 | **N better** |
| N vs T | q2_p50_us | 169.4 | 11.2 | 1397 | 30.1 | 22.7 | 141.8 | +724.4% | 54.1 | **N better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 244.6 | 45.3 | 2039 | 153.2 | 112.9 | 11.6 | +733.7% | 15.9 | **N better** |
| N vs T | c2_q1_p99_us | 1394 | 300.6 | 4313 | 571.2 | 456.4 | 1583 | +209.5% | 6.40 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c2_q2_p50_us | 166.9 | 35.3 | 2062 | 203.8 | 146.3 | 27.8 | +1135.4% | 13.0 | **N better** |
| N vs T | c4_q1_p50_us | 359.7 | 31.3 | 2512 | 211.3 | 151.0 | 180.0 | +598.4% | 14.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 1848 | 372.5 | 12508 | 7404 | 5242 | 1184 | +576.7% | 2.03 | no difference |
| N vs T | c4_q2_p50_us | 263.6 | 12.9 | 2639 | 301.1 | 213.1 | 133.7 | +901.3% | 11.1 | **N better** |
| N vs T | c4_q2_p99_us | 2073 | 643.6 | 12137 | 7415 | 5263 | 1573 | +485.5% | 1.91 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 547.3 | 45.3 | 1260 | 63.9 | 55.4 | 144.6 | +130.2% | 12.9 | **N better** |
| N vs T | write_p99_us | 1612 | 504.9 | 3541 | 1663 | 1229 | 3552 | +119.7% | 1.57 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | commits_per_s | 1553 | 185.8 | 642.8 | 105.7 | 151.1 | 157.6 | -58.6% | 6.02 | **N better** |
| N vs T | pss_mib | 86.9 | 2.38 | 2582 | 0.16 | 1.69 | 2.61 | +2870.1% | 1477 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 20852 | 3507 | -134.1 | 79.2 | 2481 | 30455 | -100.6% | 8.46 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 785.9 | 12.3 | 1386 | 50.5 | 36.7 | 164.4 | +76.4% | 16.3 | **P+ better** |
| P+ vs T | q1_p99_us | 2740 | 777.7 | 3879 | 1078 | 940.1 | 2092 | +41.6% | 1.21 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | q2_p50_us | 618.8 | 13.6 | 1397 | 30.1 | 23.3 | 222.6 | +125.7% | 33.3 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 963.1 | 142.2 | 2039 | 153.2 | 147.8 | 44.7 | +111.7% | 7.28 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 7170 | 1437 | 4313 | 571.2 | 1094 | 1866 | -39.9% | 2.61 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | c2_q2_p50_us | 765.9 | 83.5 | 2062 | 203.8 | 155.8 | 317.1 | +169.2% | 8.32 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q1_p50_us | 1984 | 69.8 | 2512 | 211.3 | 157.4 | 133.6 | +26.7% | 3.36 | **P+ better** |
| P+ vs T | c4_q1_p99_us | 10161 | 987.5 | 12508 | 7404 | 5282 | 1646 | +23.1% | 0.44 | no difference |
| P+ vs T | c4_q2_p50_us | 1765 | 142.1 | 2639 | 301.1 | 235.4 | 269.7 | +49.6% | 3.72 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 8286 | 2752 | 12137 | 7415 | 5593 | 4357 | +46.5% | 0.69 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | write_p50_us | 986.0 | 55.5 | 1260 | 63.9 | 59.8 | 165.3 | +27.8% | 4.58 | **P+ better** |
| P+ vs T | write_p99_us | 3593 | 468.8 | 3541 | 1663 | 1222 | 4085 | -1.4% | 0.04 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | commits_per_s | 833.5 | 60.4 | 642.8 | 105.7 | 86.1 | 41.2 | -22.9% | 2.22 | no difference |
| P+ vs T | pss_mib | 43.3 | 0.02 | 2582 | 0.16 | 0.11 | 0.13 | +5864.3% | 22370 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -827.0 | 169.2 | -134.1 | 79.2 | 132.1 | 1979 | -83.8% | 5.24 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 459.7 | 36.8 | 1386 | 50.5 | 44.2 | 160.3 | +201.5% | 21.0 | **H3 better** |
| H3 vs T | q1_p99_us | 1408 | 163.4 | 3879 | 1078 | 771.1 | 1335 | +175.4% | 3.20 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q2_p50_us | 467.8 | 51.8 | 1397 | 30.1 | 42.4 | 181.4 | +198.6% | 21.9 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 517.0 | 57.7 | 2039 | 153.2 | 115.7 | 43.4 | +294.4% | 13.2 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 1906 | 347.8 | 4313 | 571.2 | 472.9 | 7448 | +126.3% | 5.09 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c2_q2_p50_us | 497.8 | 73.6 | 2062 | 203.8 | 153.2 | 107.9 | +314.2% | 10.2 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 711.7 | 42.9 | 2512 | 211.3 | 152.5 | 335.2 | +253.0% | 11.8 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 3311 | 594.4 | 12508 | 7404 | 5252 | 6625 | +277.8% | 1.75 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p50_us | 684.9 | 52.5 | 2639 | 301.1 | 216.1 | 205.2 | +285.3% | 9.04 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 3796 | 693.8 | 12137 | 7415 | 5266 | 1721 | +219.7% | 1.58 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | write_p50_us | 1471 | 78.1 | 1260 | 63.9 | 71.4 | 143.0 | -14.3% | 2.96 | no difference |
| H3 vs T | write_p99_us | 4956 | 828.8 | 3541 | 1663 | 1314 | 3562 | -28.5% | 1.08 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 563.4 | 40.5 | 642.8 | 105.7 | 80.0 | 107.7 | +14.1% | 0.99 | BELOW FLOOR |
| H3 vs T | pss_mib | 105.9 | 0.41 | 2582 | 0.16 | 0.31 | 0.63 | +2338.6% | 7918 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 19851 | 3930 | -134.1 | 79.2 | 2779 | 38352 | -100.7% | 7.19 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 226.7 | 37.6 | 970.1 | 28.6 | 33.4 | 282.6 | +327.9% | 22.3 | **N better** |
| N vs H1 | q1_p99_us | 891.1 | 179.0 | 6342 | 721.2 | 525.5 | 198.7 | +611.7% | 10.4 | **N better** |
| N vs H1 | q2_p50_us | 169.4 | 11.2 | 791.9 | 56.4 | 40.7 | 48.3 | +367.4% | 15.3 | **N better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 30423 | 1216 | 580.6 | 119.2 | 863.6 | 4069 | -98.1% | 34.6 | **H1 better** |
| N vs H1 | q6_p50_us | 47610 | 3020 | 7564 | 550.0 | 2170 | 9377 | -84.1% | 18.5 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 244.6 | 45.3 | 1087 | 71.7 | 60.0 | 112.4 | +344.5% | 14.1 | **N better** |
| N vs H1 | c2_q1_p99_us | 1394 | 300.6 | 5492 | 1150 | 840.8 | 407.0 | +294.1% | 4.87 | **N better** |
| N vs H1 | c2_q2_p50_us | 166.9 | 35.3 | 1002 | 55.2 | 46.4 | 152.4 | +500.1% | 18.0 | **N better** |
| N vs H1 | c4_q1_p50_us | 359.7 | 31.3 | 1098 | 126.5 | 92.1 | 170.0 | +205.3% | 8.02 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 1848 | 372.5 | 4986 | 685.2 | 551.5 | 462.0 | +169.7% | 5.69 | **N better** |
| N vs H1 | c4_q2_p50_us | 263.6 | 12.9 | 987.5 | 101.4 | 72.2 | 112.6 | +274.6% | 10.0 | **N better** |
| N vs H1 | c4_q2_p99_us | 2073 | 643.6 | 7205 | 1352 | 1059 | 1952 | +247.6% | 4.85 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | write_p50_us | 547.3 | 45.3 | 1698 | 132.5 | 99.0 | 153.3 | +210.3% | 11.6 | **N better** |
| N vs H1 | write_p99_us | 1612 | 504.9 | 5485 | 1053 | 826.0 | 392.0 | +240.2% | 4.69 | **N better** |
| N vs H1 | commits_per_s | 1553 | 185.8 | 497.8 | 25.3 | 132.6 | 167.3 | -67.9% | 7.96 | **N better** |
| N vs H1 | pss_mib | 86.9 | 2.38 | 428.4 | 3.50 | 3.00 | 37.3 | +392.8% | 114.0 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 20852 | 3507 | 21890 | 3512 | 3510 | 34826 | +5.0% | 0.30 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| P+ vs H1 | q1_p50_us | 785.9 | 12.3 | 970.1 | 28.6 | 22.0 | 288.7 | +23.4% | 8.38 | BELOW FLOOR |
| P+ vs H1 | q1_p99_us | 2740 | 777.7 | 6342 | 721.2 | 750.0 | 1992 | +131.5% | 4.80 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q2_p50_us | 618.8 | 13.6 | 791.9 | 56.4 | 41.0 | 178.4 | +28.0% | 4.22 | BELOW FLOOR |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 360.7 | 36.5 | 961.3 | 90.8 | 69.2 | 38.0 | +166.5% | 8.68 | **P+ better** |
| P+ vs H1 | q4_p50_us | 13617 | 452.5 | 25815 | 2824 | 2022 | 5803 | +89.6% | 6.03 | **P+ better** |
| P+ vs H1 | q5_p50_us | 40758 | 1901 | 580.6 | 119.2 | 1347 | 1607 | -98.6% | 29.8 | **H1 better** |
| P+ vs H1 | q6_p50_us | 40997 | 3698 | 7564 | 550.0 | 2643 | 18489 | -81.5% | 12.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c2_q1_p50_us | 963.1 | 142.2 | 1087 | 71.7 | 112.6 | 120.4 | +12.9% | 1.10 | no difference |
| P+ vs H1 | c2_q1_p99_us | 7170 | 1437 | 5492 | 1150 | 1302 | 1068 | -23.4% | 1.29 | no difference |
| P+ vs H1 | c2_q2_p50_us | 765.9 | 83.5 | 1002 | 55.2 | 70.8 | 350.7 | +30.8% | 3.33 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q1_p50_us | 1984 | 69.8 | 1098 | 126.5 | 102.2 | 119.7 | -44.6% | 8.66 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 10161 | 987.5 | 4986 | 685.2 | 849.9 | 1233 | -50.9% | 6.09 | **H1 better** |
| P+ vs H1 | c4_q2_p50_us | 1765 | 142.1 | 987.5 | 101.4 | 123.4 | 259.9 | -44.0% | 6.30 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 8286 | 2752 | 7205 | 1352 | 2168 | 4508 | -13.0% | 0.50 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | write_p50_us | 986.0 | 55.5 | 1698 | 132.5 | 101.5 | 173.0 | +72.2% | 7.01 | **P+ better** |
| P+ vs H1 | write_p99_us | 3593 | 468.8 | 5485 | 1053 | 815.3 | 2055 | +52.7% | 2.32 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | commits_per_s | 833.5 | 60.4 | 497.8 | 25.3 | 46.3 | 69.7 | -40.3% | 7.25 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.02 | 428.4 | 3.50 | 2.48 | 37.2 | +889.6% | 155.5 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -827.0 | 169.2 | 21890 | 3512 | 2486 | 17007 | -2746.9% | 9.14 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 226.7 | 37.6 | 106.1 | 8.18 | 27.2 | 51.1 | -53.2% | 4.43 | **H2 better** |
| N vs H2 | q1_p99_us | 891.1 | 179.0 | 795.6 | 281.0 | 235.6 | 507.6 | -10.7% | 0.41 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q2_p50_us | 169.4 | 11.2 | 222.8 | 25.7 | 19.8 | 57.3 | +31.5% | 2.69 | BELOW FLOOR |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 30423 | 1216 | 153.3 | 25.0 | 859.7 | 4069 | -99.5% | 35.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | 47610 | 3020 | 3138 | 614.9 | 2179 | 9369 | -93.4% | 20.4 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 244.6 | 45.3 | 106.0 | 35.7 | 40.8 | 95.5 | -56.7% | 3.40 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q1_p99_us | 1394 | 300.6 | 2122 | 654.4 | 509.2 | 166.3 | +52.3% | 1.43 | no difference |
| N vs H2 | c2_q2_p50_us | 166.9 | 35.3 | 264.5 | 34.1 | 34.7 | 157.4 | +58.5% | 2.81 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p50_us | 359.7 | 31.3 | 109.1 | 54.8 | 44.6 | 173.2 | -69.7% | 5.61 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q1_p99_us | 1848 | 372.5 | 2620 | 348.8 | 360.8 | 1060 | +41.8% | 2.14 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p50_us | 263.6 | 12.9 | 253.4 | 64.1 | 46.2 | 54.6 | -3.8% | 0.22 | BELOW FLOOR |
| N vs H2 | c4_q2_p99_us | 2073 | 643.6 | 4235 | 961.0 | 817.9 | 1530 | +104.3% | 2.64 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 547.3 | 45.3 | 1429 | 82.1 | 66.3 | 385.1 | +161.2% | 13.3 | **N better** |
| N vs H2 | write_p99_us | 1612 | 504.9 | 4819 | 549.3 | 527.5 | 364.2 | +198.9% | 6.08 | **N better** |
| N vs H2 | commits_per_s | 1553 | 185.8 | 564.8 | 53.4 | 136.7 | 180.9 | -63.6% | 7.23 | **N better** |
| N vs H2 | pss_mib | 86.9 | 2.38 | 43.5 | 0.02 | 1.69 | 2.61 | -50.0% | 25.8 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 20852 | 3507 | -767.6 | 157.5 | 2483 | 30499 | -103.7% | 8.71 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 785.9 | 12.3 | 106.1 | 8.18 | 10.4 | 78.1 | -86.5% | 65.2 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2740 | 777.7 | 795.6 | 281.0 | 584.7 | 2046 | -71.0% | 3.33 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | q2_p50_us | 618.8 | 13.6 | 222.8 | 25.7 | 20.6 | 181.0 | -64.0% | 19.2 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 360.7 | 36.5 | 286.9 | 81.5 | 63.1 | 296.2 | -20.5% | 1.17 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q4_p50_us | 13617 | 452.5 | 831.5 | 95.8 | 327.1 | 2452 | -93.9% | 39.1 | **H2 better** |
| P+ vs H2 | q5_p50_us | 40758 | 1901 | 153.3 | 25.0 | 1344 | 1607 | -99.6% | 30.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q6_p50_us | 40997 | 3698 | 3138 | 614.9 | 2650 | 18485 | -92.3% | 14.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q1_p50_us | 963.1 | 142.2 | 106.0 | 35.7 | 103.6 | 104.8 | -89.0% | 8.27 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q1_p99_us | 7170 | 1437 | 2122 | 654.4 | 1117 | 1001 | -70.4% | 4.52 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 765.9 | 83.5 | 264.5 | 34.1 | 63.8 | 352.9 | -65.5% | 7.86 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c4_q1_p50_us | 1984 | 69.8 | 109.1 | 54.8 | 62.8 | 124.2 | -94.5% | 29.9 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 10161 | 987.5 | 2620 | 348.8 | 740.6 | 1559 | -74.2% | 10.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p50_us | 1765 | 142.1 | 253.4 | 64.1 | 110.2 | 240.5 | -85.6% | 13.7 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 8286 | 2752 | 4235 | 961.0 | 2061 | 4341 | -48.9% | 1.97 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | write_p50_us | 986.0 | 55.5 | 1429 | 82.1 | 70.1 | 393.3 | +45.0% | 6.33 | **P+ better** |
| P+ vs H2 | write_p99_us | 3593 | 468.8 | 4819 | 549.3 | 510.6 | 2049 | +34.1% | 2.40 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 833.5 | 60.4 | 564.8 | 53.4 | 57.0 | 97.8 | -32.2% | 4.71 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.02 | 43.5 | 0.02 | 0.02 | 0.13 | +0.4% | 8.95 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -827.0 | 169.2 | -767.6 | 157.5 | 163.5 | 2567 | -7.2% | 0.36 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 254.7 | 24.9 | 822.8 | 33.0 | 29.2 | 47.7 | +223.1% | 19.4 | **N better** |
| N vs P+ | q1_p99_us | 1113 | 264.1 | 2536 | 535.8 | 422.4 | 625.5 | +127.8% | 3.37 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 176.7 | 19.2 | 628.7 | 23.5 | 21.5 | 48.1 | +255.9% | 21.1 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 29642 | 1243 | 44392 | 4246 | 3128 | 3359 | +49.8% | 4.72 | **N better** |
| N vs P+ | q6_p50_us | 51231 | 6842 | 44350 | 1833 | 5008 | 11665 | -13.4% | 1.37 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 213.3 | 51.4 | 1049 | 174.1 | 128.4 | 108.9 | +392.0% | 6.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 1269 | 104.0 | 7422 | 1453 | 1030 | 3828 | +484.9% | 5.97 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 174.7 | 8.53 | 797.8 | 145.3 | 102.9 | 77.6 | +356.6% | 6.05 | **N better** |
| N vs P+ | c4_q1_p50_us | 472.2 | 69.9 | 2036 | 108.1 | 91.1 | 517.7 | +331.1% | 17.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1661 | 194.2 | 10294 | 797.9 | 580.7 | 4202 | +519.6% | 14.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 356.5 | 74.0 | 1867 | 83.6 | 78.9 | 423.6 | +423.7% | 19.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1896 | 591.2 | 10076 | 2921 | 2107 | 566.9 | +431.5% | 3.88 | **N better** |
| N vs P+ | write_p50_us | 548.3 | 77.3 | 1054 | 90.6 | 84.2 | 44.2 | +92.2% | 6.00 | **N better** |
| N vs P+ | write_p99_us | 1913 | 542.6 | 3303 | 373.3 | 465.7 | 2422 | +72.7% | 2.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1446 | 213.8 | 750.4 | 27.1 | 152.4 | 325.0 | -48.1% | 4.56 | **N better** |
| N vs P+ | pss_mib | 86.8 | 3.15 | 43.3 | 0.02 | 2.23 | 0.58 | -50.1% | 19.5 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 20677 | 3664 | -806.4 | 171.0 | 2594 | 42159 | -103.9% | 8.28 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 254.7 | 24.9 | 95.9 | 13.9 | 20.2 | 27.8 | -62.4% | 7.87 | **P better** |
| N vs P | q1_p99_us | 1113 | 264.1 | 682.4 | 193.8 | 231.6 | 1577 | -38.7% | 1.86 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 176.7 | 19.2 | 203.4 | 16.5 | 17.9 | 22.0 | +15.1% | 1.49 | no difference |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 29642 | 1243 | 162.2 | 38.2 | 879.6 | 3108 | -99.5% | 33.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 51231 | 6842 | 3464 | 454.0 | 4849 | 4959 | -93.2% | 9.85 | **P better** |
| N vs P | c2_q1_p50_us | 213.3 | 51.4 | 101.4 | 28.8 | 41.7 | 84.8 | -52.5% | 2.69 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 1269 | 104.0 | 1798 | 740.0 | 528.4 | 748.5 | +41.7% | 1.00 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 174.7 | 8.53 | 212.4 | 60.8 | 43.4 | 64.5 | +21.6% | 0.87 | BELOW FLOOR |
| N vs P | c4_q1_p50_us | 472.2 | 69.9 | 147.2 | 30.3 | 53.9 | 124.8 | -68.8% | 6.03 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1661 | 194.2 | 2868 | 436.1 | 337.6 | 205.7 | +72.6% | 3.57 | **N better** |
| N vs P | c4_q2_p50_us | 356.5 | 74.0 | 353.3 | 73.7 | 73.8 | 186.1 | -0.9% | 0.04 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 1896 | 591.2 | 4067 | 1517 | 1151 | 374.3 | +114.5% | 1.89 | no difference |
| N vs P | write_p50_us | 548.3 | 77.3 | 123022 | 2796 | 1978 | 2195 | +22336.4% | 61.9 | **N better** |
| N vs P | write_p99_us | 1913 | 542.6 | 168101 | 7237 | 5132 | 47369 | +8689.3% | 32.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1446 | 213.8 | 7.88 | 0.11 | 151.2 | 320.6 | -99.5% | 9.51 | **N better** |
| N vs P | pss_mib | 86.8 | 3.15 | 43.5 | 0.05 | 2.23 | 0.56 | -49.9% | 19.4 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 20677 | 3664 | -810.5 | 213.6 | 2595 | 41990 | -103.9% | 8.28 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 822.8 | 33.0 | 95.9 | 13.9 | 25.3 | 40.4 | -88.4% | 28.7 | **P better** |
| P+ vs P | q1_p99_us | 2536 | 535.8 | 682.4 | 193.8 | 402.9 | 1462 | -73.1% | 4.60 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 628.7 | 23.5 | 203.4 | 16.5 | 20.3 | 49.9 | -67.7% | 21.0 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 371.0 | 22.9 | 312.1 | 38.1 | 31.4 | 77.7 | -15.9% | 1.87 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 14008 | 771.8 | 824.4 | 126.8 | 553.0 | 1210 | -94.1% | 23.8 | **P better** |
| P+ vs P | q5_p50_us | 44392 | 4246 | 162.2 | 38.2 | 3002 | 1279 | -99.6% | 14.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 44350 | 1833 | 3464 | 454.0 | 1335 | 10560 | -92.2% | 30.6 | **P better** |
| P+ vs P | c2_q1_p50_us | 1049 | 174.1 | 101.4 | 28.8 | 124.8 | 76.4 | -90.3% | 7.59 | **P better** |
| P+ vs P | c2_q1_p99_us | 7422 | 1453 | 1798 | 740.0 | 1153 | 3897 | -75.8% | 4.88 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 797.8 | 145.3 | 212.4 | 60.8 | 111.4 | 70.8 | -73.4% | 5.25 | **P better** |
| P+ vs P | c4_q1_p50_us | 2036 | 108.1 | 147.2 | 30.3 | 79.4 | 506.5 | -92.8% | 23.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 10294 | 797.9 | 2868 | 436.1 | 643.0 | 4197 | -72.1% | 11.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1867 | 83.6 | 353.3 | 73.7 | 78.8 | 420.7 | -81.1% | 19.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 10076 | 2921 | 4067 | 1517 | 2327 | 449.3 | -59.6% | 2.58 | no difference |
| P+ vs P | write_p50_us | 1054 | 90.6 | 123022 | 2796 | 1978 | 2195 | +11574.2% | 61.7 | **P+ better** |
| P+ vs P | write_p99_us | 3303 | 373.3 | 168101 | 7237 | 5124 | 47310 | +4989.3% | 32.2 | **P+ better** |
| P+ vs P | commits_per_s | 750.4 | 27.1 | 7.88 | 0.11 | 19.2 | 53.1 | -99.0% | 38.7 | **P+ better** |
| P+ vs P | pss_mib | 43.3 | 0.02 | 43.5 | 0.05 | 0.03 | 0.13 | +0.3% | 4.25 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -806.4 | 171.0 | -810.5 | 213.6 | 193.5 | 4454 | +0.5% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 95.9 | 13.9 | 99.3 | 17.6 | 15.9 | 48.4 | +3.6% | 0.22 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q1_p99_us | 682.4 | 193.8 | 1127 | 403.5 | 316.5 | 1457 | +65.1% | 1.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 203.4 | 16.5 | 222.9 | 27.9 | 22.9 | 127.5 | +9.6% | 0.85 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 312.1 | 38.1 | 345.8 | 75.7 | 59.9 | 107.5 | +10.8% | 0.56 | BELOW FLOOR |
| P vs M | q4_p50_us | 824.4 | 126.8 | 796.3 | 135.8 | 131.4 | 26.3 | -3.4% | 0.21 | no difference |
| P vs M | q5_p50_us | 162.2 | 38.2 | 147.4 | 44.8 | 41.7 | 138.8 | -9.1% | 0.36 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | 3464 | 454.0 | 3180 | 450.1 | 452.1 | 144.3 | -8.2% | 0.63 | no difference |
| P vs M | c2_q1_p50_us | 101.4 | 28.8 | 103.6 | 17.3 | 23.8 | 81.4 | +2.2% | 0.09 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1798 | 740.0 | 2315 | 1058 | 913.2 | 819.8 | +28.7% | 0.57 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 212.4 | 60.8 | 234.9 | 37.2 | 50.4 | 65.5 | +10.6% | 0.45 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 147.2 | 30.3 | 135.0 | 20.1 | 25.7 | 74.7 | -8.3% | 0.48 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2868 | 436.1 | 2855 | 247.5 | 354.6 | 340.2 | -0.4% | 0.04 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 353.3 | 73.7 | 307.9 | 63.2 | 68.6 | 175.4 | -12.9% | 0.66 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p99_us | 4067 | 1517 | 4741 | 1254 | 1392 | 416.1 | +16.6% | 0.48 | no difference |
| P vs M | write_p50_us | 123022 | 2796 | 122702 | 3474 | 3153 | 22704 | -0.3% | 0.10 | BELOW FLOOR |
| P vs M | write_p99_us | 168101 | 7237 | 174767 | 6547 | 6900 | 55968 | +4.0% | 0.97 | BELOW FLOOR |
| P vs M | commits_per_s | 7.88 | 0.11 | 7.92 | 0.26 | 0.20 | 0.73 | +0.5% | 0.21 | BELOW FLOOR |
| P vs M | pss_mib | 43.5 | 0.05 | 43.4 | 0.05 | 0.05 | 0.03 | -0.2% | 1.36 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -810.5 | 213.6 | -768.1 | 164.4 | 190.6 | 2456 | -5.2% | 0.22 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 254.7 | 24.9 | 458.9 | 66.3 | 50.1 | 36.5 | +80.2% | 4.07 | **N better** |
| N vs H3 | q1_p99_us | 1113 | 264.1 | 1418 | 310.2 | 288.1 | 615.7 | +27.3% | 1.06 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 176.7 | 19.2 | 476.3 | 67.9 | 49.9 | 12.5 | +169.6% | 6.00 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 29642 | 1243 | 44502 | 3178 | 2413 | 5145 | +50.1% | 6.16 | **N better** |
| N vs H3 | q6_p50_us | 51231 | 6842 | 47078 | 6437 | 6643 | 8086 | -8.1% | 0.63 | BELOW FLOOR |
| N vs H3 | c2_q1_p50_us | 213.3 | 51.4 | 493.2 | 56.6 | 54.0 | 90.3 | +131.3% | 5.18 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 1269 | 104.0 | 2693 | 286.7 | 215.6 | 1555 | +112.2% | 6.60 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c2_q2_p50_us | 174.7 | 8.53 | 441.4 | 68.2 | 48.6 | 71.8 | +152.6% | 5.49 | **N better** |
| N vs H3 | c4_q1_p50_us | 472.2 | 69.9 | 594.1 | 88.6 | 79.8 | 334.3 | +25.8% | 1.53 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p99_us | 1661 | 194.2 | 2674 | 575.6 | 429.5 | 4171 | +61.0% | 2.36 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 356.5 | 74.0 | 583.6 | 65.0 | 69.6 | 153.1 | +63.7% | 3.26 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1896 | 591.2 | 3410 | 1120 | 895.4 | 7727 | +79.9% | 1.69 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 548.3 | 77.3 | 1439 | 66.7 | 72.2 | 56.0 | +162.5% | 12.3 | **N better** |
| N vs H3 | write_p99_us | 1913 | 542.6 | 5235 | 826.5 | 699.1 | 2657 | +173.7% | 4.75 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1446 | 213.8 | 522.5 | 35.9 | 153.3 | 322.9 | -63.9% | 6.02 | **N better** |
| N vs H3 | pss_mib | 86.8 | 3.15 | 111.8 | 0.12 | 2.23 | 0.60 | +28.9% | 11.2 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 20677 | 3664 | 21941 | 4217 | 3950 | 54682 | +6.1% | 0.32 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 822.8 | 33.0 | 458.9 | 66.3 | 52.4 | 46.8 | -44.2% | 6.95 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2536 | 535.8 | 1418 | 310.2 | 437.8 | 174.7 | -44.1% | 2.55 | no difference |
| P+ vs H3 | q2_p50_us | 628.7 | 23.5 | 476.3 | 67.9 | 50.8 | 46.5 | -24.2% | 3.00 | no difference |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 371.0 | 22.9 | 447.8 | 35.1 | 29.7 | 62.4 | +20.7% | 2.59 | no difference |
| P+ vs H3 | q4_p50_us | 14008 | 771.8 | 14171 | 831.5 | 802.2 | 1828 | +1.2% | 0.20 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 44392 | 4246 | 44502 | 3178 | 3750 | 4295 | +0.2% | 0.03 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 44350 | 1833 | 47078 | 6437 | 4733 | 12341 | +6.2% | 0.58 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 1049 | 174.1 | 493.2 | 56.6 | 129.5 | 82.5 | -53.0% | 4.29 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 7422 | 1453 | 2693 | 286.7 | 1047 | 4128 | -63.7% | 4.52 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 797.8 | 145.3 | 441.4 | 68.2 | 113.5 | 77.4 | -44.7% | 3.14 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 2036 | 108.1 | 594.1 | 88.6 | 98.8 | 593.9 | -70.8% | 14.6 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 10294 | 797.9 | 2674 | 575.6 | 695.7 | 5913 | -74.0% | 11.0 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1867 | 83.6 | 583.6 | 65.0 | 74.9 | 407.2 | -68.7% | 17.1 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 10076 | 2921 | 3410 | 1120 | 2212 | 7731 | -66.2% | 3.01 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p50_us | 1054 | 90.6 | 1439 | 66.7 | 79.6 | 63.4 | +36.6% | 4.85 | **P+ better** |
| P+ vs H3 | write_p99_us | 3303 | 373.3 | 5235 | 826.5 | 641.3 | 1188 | +58.5% | 3.01 | **P+ better** |
| P+ vs H3 | commits_per_s | 750.4 | 27.1 | 522.5 | 35.9 | 31.8 | 65.6 | -30.4% | 7.16 | **P+ better** |
| P+ vs H3 | pss_mib | 43.3 | 0.02 | 111.8 | 0.12 | 0.09 | 0.25 | +158.1% | 775.0 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -806.4 | 171.0 | 21941 | 4217 | 2985 | 35310 | -2821.0% | 7.62 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 254.7 | 24.9 | 1428 | 31.2 | 28.2 | 107.9 | +460.8% | 41.6 | **N better** |
| N vs T | q1_p99_us | 1113 | 264.1 | 3849 | 999.3 | 730.8 | 802.0 | +245.8% | 3.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q2_p50_us | 176.7 | 19.2 | 1451 | 46.1 | 35.3 | 30.9 | +721.6% | 36.1 | **N better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 213.3 | 51.4 | 2306 | 237.5 | 171.8 | 161.0 | +981.3% | 12.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c2_q1_p99_us | 1269 | 104.0 | 7790 | 3199 | 2263 | 1643 | +513.9% | 2.88 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c2_q2_p50_us | 174.7 | 8.53 | 2326 | 246.6 | 174.4 | 56.0 | +1231.5% | 12.3 | **N better** |
| N vs T | c4_q1_p50_us | 472.2 | 69.9 | 2823 | 195.7 | 146.9 | 123.4 | +497.8% | 16.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 1661 | 194.2 | 6761 | 1321 | 944.3 | 3081 | +306.9% | 5.40 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | c4_q2_p50_us | 356.5 | 74.0 | 2928 | 249.1 | 183.7 | 382.4 | +721.4% | 14.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p99_us | 1896 | 591.2 | 7121 | 1700 | 1273 | 3145 | +275.6% | 4.11 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 548.3 | 77.3 | 1295 | 84.8 | 81.2 | 111.9 | +136.2% | 9.20 | **N better** |
| N vs T | write_p99_us | 1913 | 542.6 | 4151 | 1862 | 1372 | 2472 | +117.0% | 1.63 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | commits_per_s | 1446 | 213.8 | 660.5 | 33.8 | 153.0 | 371.8 | -54.3% | 5.13 | **N better** |
| N vs T | pss_mib | 86.8 | 3.15 | 2582 | 0.15 | 2.23 | 0.56 | +2876.3% | 1118 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 20677 | 3664 | -166.3 | 80.2 | 2592 | 41961 | -100.8% | 8.04 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 822.8 | 33.0 | 1428 | 31.2 | 32.1 | 111.8 | +73.6% | 18.9 | **P+ better** |
| P+ vs T | q1_p99_us | 2536 | 535.8 | 3849 | 999.3 | 801.8 | 542.7 | +51.8% | 1.64 | no difference |
| P+ vs T | q2_p50_us | 628.7 | 23.5 | 1451 | 46.1 | 36.6 | 54.4 | +130.9% | 22.5 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 1049 | 174.1 | 2306 | 237.5 | 208.3 | 156.7 | +119.8% | 6.04 | **P+ better** |
| P+ vs T | c2_q1_p99_us | 7422 | 1453 | 7790 | 3199 | 2484 | 4162 | +5.0% | 0.15 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c2_q2_p50_us | 797.8 | 145.3 | 2326 | 246.6 | 202.4 | 63.1 | +191.6% | 7.55 | **P+ better** |
| P+ vs T | c4_q1_p50_us | 2036 | 108.1 | 2823 | 195.7 | 158.1 | 506.2 | +38.7% | 4.98 | **P+ better** |
| P+ vs T | c4_q1_p99_us | 10294 | 797.9 | 6761 | 1321 | 1091 | 5203 | -34.3% | 3.24 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | c4_q2_p50_us | 1867 | 83.6 | 2928 | 249.1 | 185.8 | 537.2 | +56.9% | 5.71 | **P+ better** |
| P+ vs T | c4_q2_p99_us | 10076 | 2921 | 7121 | 1700 | 2389 | 3155 | -29.3% | 1.24 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | write_p50_us | 1054 | 90.6 | 1295 | 84.8 | 87.8 | 115.8 | +22.9% | 2.75 | no difference |
| P+ vs T | write_p99_us | 3303 | 373.3 | 4151 | 1862 | 1343 | 678.6 | +25.7% | 0.63 | REFUSED (warm-up MAD above 15% of the median on T) |
| P+ vs T | commits_per_s | 750.4 | 27.1 | 660.5 | 33.8 | 30.7 | 195.6 | -12.0% | 2.94 | BELOW FLOOR |
| P+ vs T | pss_mib | 43.3 | 0.02 | 2582 | 0.15 | 0.11 | 0.14 | +5861.7% | 23992 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -806.4 | 171.0 | -166.3 | 80.2 | 133.6 | 4178 | -79.4% | 4.79 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 458.9 | 66.3 | 1428 | 31.2 | 51.8 | 107.5 | +211.2% | 18.7 | **H3 better** |
| H3 vs T | q1_p99_us | 1418 | 310.2 | 3849 | 999.3 | 739.8 | 531.5 | +171.5% | 3.29 | **H3 better** |
| H3 vs T | q2_p50_us | 476.3 | 67.9 | 1451 | 46.1 | 58.0 | 28.3 | +204.7% | 16.8 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 493.2 | 56.6 | 2306 | 237.5 | 172.7 | 144.4 | +367.5% | 10.5 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 2693 | 286.7 | 7790 | 3199 | 2271 | 2256 | +189.3% | 2.24 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c2_q2_p50_us | 441.4 | 68.2 | 2326 | 246.6 | 180.9 | 55.8 | +427.0% | 10.4 | **H3 better** |
| H3 vs T | c4_q1_p50_us | 594.1 | 88.6 | 2823 | 195.7 | 151.9 | 316.1 | +375.2% | 14.7 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p99_us | 2674 | 575.6 | 6761 | 1321 | 1019 | 5177 | +152.8% | 4.01 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | c4_q2_p50_us | 583.6 | 65.0 | 2928 | 249.1 | 182.0 | 364.1 | +401.8% | 12.9 | **H3 better** |
| H3 vs T | c4_q2_p99_us | 3410 | 1120 | 7121 | 1700 | 1439 | 8327 | +108.8% | 2.58 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | write_p50_us | 1439 | 66.7 | 1295 | 84.8 | 76.3 | 120.8 | -10.0% | 1.89 | no difference |
| H3 vs T | write_p99_us | 5235 | 826.5 | 4151 | 1862 | 1441 | 1286 | -20.7% | 0.75 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | commits_per_s | 522.5 | 35.9 | 660.5 | 33.8 | 34.9 | 192.2 | +26.4% | 3.96 | BELOW FLOOR |
| H3 vs T | pss_mib | 111.8 | 0.12 | 2582 | 0.15 | 0.14 | 0.22 | +2209.5% | 18091 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 21941 | 4217 | -166.3 | 80.2 | 2983 | 35074 | -100.8% | 7.41 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 254.7 | 24.9 | 1009 | 72.1 | 53.9 | 153.7 | +296.1% | 14.0 | **N better** |
| N vs H1 | q1_p99_us | 1113 | 264.1 | 5731 | 933.6 | 686.0 | 611.0 | +414.9% | 6.73 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p50_us | 176.7 | 19.2 | 794.2 | 30.8 | 25.7 | 32.4 | +349.6% | 24.0 | **N better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 29642 | 1243 | 672.8 | 175.9 | 887.9 | 32622 | -97.7% | 32.6 | BELOW FLOOR |
| N vs H1 | q6_p50_us | 51231 | 6842 | 6432 | 409.9 | 4847 | 5012 | -87.4% | 9.24 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 213.3 | 51.4 | 1073 | 26.4 | 40.8 | 109.2 | +403.1% | 21.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c2_q1_p99_us | 1269 | 104.0 | 7343 | 527.6 | 380.2 | 2181 | +478.7% | 16.0 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c2_q2_p50_us | 174.7 | 8.53 | 1005 | 88.3 | 62.8 | 61.9 | +475.5% | 13.2 | **N better** |
| N vs H1 | c4_q1_p50_us | 472.2 | 69.9 | 1081 | 38.1 | 56.3 | 209.8 | +128.9% | 10.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 1661 | 194.2 | 4605 | 1016 | 731.6 | 1955 | +177.2% | 4.02 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c4_q2_p50_us | 356.5 | 74.0 | 1032 | 69.4 | 71.7 | 143.1 | +189.5% | 9.42 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p99_us | 1896 | 591.2 | 4009 | 1188 | 938.2 | 735.7 | +111.5% | 2.25 | no difference |
| N vs H1 | write_p50_us | 548.3 | 77.3 | 1883 | 106.4 | 93.0 | 156.7 | +243.5% | 14.4 | **N better** |
| N vs H1 | write_p99_us | 1913 | 542.6 | 4901 | 511.8 | 527.4 | 2488 | +156.2% | 5.67 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | commits_per_s | 1446 | 213.8 | 468.9 | 21.8 | 151.9 | 343.9 | -67.6% | 6.43 | **N better** |
| N vs H1 | pss_mib | 86.8 | 3.15 | 430.7 | 6.19 | 4.92 | 10.7 | +396.4% | 70.0 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 20677 | 3664 | 24565 | 3583 | 3624 | 42006 | +18.8% | 1.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| P+ vs H1 | q1_p50_us | 822.8 | 33.0 | 1009 | 72.1 | 56.0 | 156.4 | +22.6% | 3.32 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2536 | 535.8 | 5731 | 933.6 | 761.1 | 157.3 | +126.0% | 4.20 | **P+ better** |
| P+ vs H1 | q2_p50_us | 628.7 | 23.5 | 794.2 | 30.8 | 27.4 | 55.3 | +26.3% | 6.04 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 371.0 | 22.9 | 973.2 | 68.8 | 51.3 | 213.1 | +162.3% | 11.7 | **P+ better** |
| P+ vs H1 | q4_p50_us | 14008 | 771.8 | 27379 | 2288 | 1707 | 3221 | +95.4% | 7.83 | **P+ better** |
| P+ vs H1 | q5_p50_us | 44392 | 4246 | 672.8 | 175.9 | 3005 | 32499 | -98.5% | 14.6 | **H1 better** |
| P+ vs H1 | q6_p50_us | 44350 | 1833 | 6432 | 409.9 | 1328 | 10585 | -85.5% | 28.6 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 1049 | 174.1 | 1073 | 26.4 | 124.5 | 102.8 | +2.3% | 0.19 | BELOW FLOOR |
| P+ vs H1 | c2_q1_p99_us | 7422 | 1453 | 7343 | 527.6 | 1093 | 4402 | -1.1% | 0.07 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c2_q2_p50_us | 797.8 | 145.3 | 1005 | 88.3 | 120.3 | 68.3 | +26.0% | 1.73 | no difference |
| P+ vs H1 | c4_q1_p50_us | 2036 | 108.1 | 1081 | 38.1 | 81.1 | 533.8 | -46.9% | 11.8 | **H1 better** |
| P+ vs H1 | c4_q1_p99_us | 10294 | 797.9 | 4605 | 1016 | 913.6 | 4625 | -55.3% | 6.23 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| P+ vs H1 | c4_q2_p50_us | 1867 | 83.6 | 1032 | 69.4 | 76.9 | 403.5 | -44.7% | 10.9 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 10076 | 2921 | 4009 | 1188 | 2229 | 776.6 | -60.2% | 2.72 | no difference |
| P+ vs H1 | write_p50_us | 1054 | 90.6 | 1883 | 106.4 | 98.8 | 159.5 | +78.7% | 8.39 | **P+ better** |
| P+ vs H1 | write_p99_us | 3303 | 373.3 | 4901 | 511.8 | 447.9 | 733.8 | +48.4% | 3.57 | **P+ better** |
| P+ vs H1 | commits_per_s | 750.4 | 27.1 | 468.9 | 21.8 | 24.6 | 135.4 | -37.5% | 11.4 | **P+ better** |
| P+ vs H1 | pss_mib | 43.3 | 0.02 | 430.7 | 6.19 | 4.38 | 10.7 | +894.4% | 88.4 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -806.4 | 171.0 | 24565 | 3583 | 2537 | 4604 | -3146.5% | 10.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs H2 | q1_p50_us | 254.7 | 24.9 | 107.8 | 7.22 | 18.3 | 26.9 | -57.7% | 8.01 | **H2 better** |
| N vs H2 | q1_p99_us | 1113 | 264.1 | 737.6 | 263.8 | 263.9 | 984.4 | -33.7% | 1.42 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | q2_p50_us | 176.7 | 19.2 | 245.0 | 28.0 | 24.0 | 18.8 | +38.7% | 2.84 | no difference |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 29642 | 1243 | 157.3 | 25.3 | 879.4 | 3108 | -99.5% | 33.5 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | 51231 | 6842 | 2883 | 460.4 | 4849 | 4994 | -94.4% | 9.97 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 213.3 | 51.4 | 112.2 | 28.0 | 41.4 | 81.5 | -47.4% | 2.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q1_p99_us | 1269 | 104.0 | 2400 | 792.3 | 565.0 | 1375 | +89.2% | 2.00 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q2_p50_us | 174.7 | 8.53 | 240.3 | 50.3 | 36.1 | 56.9 | +37.5% | 1.82 | no difference |
| N vs H2 | c4_q1_p50_us | 472.2 | 69.9 | 133.0 | 36.3 | 55.7 | 125.4 | -71.8% | 6.09 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q1_p99_us | 1661 | 194.2 | 2526 | 518.6 | 391.5 | 536.6 | +52.0% | 2.21 | no difference |
| N vs H2 | c4_q2_p50_us | 356.5 | 74.0 | 336.3 | 78.8 | 76.4 | 172.3 | -5.7% | 0.26 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q2_p99_us | 1896 | 591.2 | 3788 | 1268 | 989.0 | 362.8 | +99.8% | 1.91 | no difference |
| N vs H2 | write_p50_us | 548.3 | 77.3 | 1485 | 62.9 | 70.5 | 244.6 | +170.9% | 13.3 | **N better** |
| N vs H2 | write_p99_us | 1913 | 542.6 | 5029 | 546.5 | 544.6 | 2456 | +163.0% | 5.72 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1446 | 213.8 | 558.5 | 42.1 | 154.1 | 334.5 | -61.4% | 5.76 | **N better** |
| N vs H2 | pss_mib | 86.8 | 3.15 | 43.5 | 0.03 | 2.23 | 0.57 | -49.9% | 19.4 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 20677 | 3664 | -824.2 | 181.9 | 2594 | 42004 | -104.0% | 8.29 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 822.8 | 33.0 | 107.8 | 7.22 | 23.9 | 39.9 | -86.9% | 30.0 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2536 | 535.8 | 737.6 | 263.8 | 422.3 | 787.6 | -70.9% | 4.26 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p50_us | 628.7 | 23.5 | 245.0 | 28.0 | 25.9 | 48.5 | -61.0% | 14.8 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 371.0 | 22.9 | 349.5 | 26.7 | 24.8 | 129.7 | -5.8% | 0.87 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q4_p50_us | 14008 | 771.8 | 794.5 | 88.7 | 549.3 | 1222 | -94.3% | 24.1 | **H2 better** |
| P+ vs H2 | q5_p50_us | 44392 | 4246 | 157.3 | 25.3 | 3002 | 1277 | -99.6% | 14.7 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q6_p50_us | 44350 | 1833 | 2883 | 460.4 | 1336 | 10576 | -93.5% | 31.0 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 1049 | 174.1 | 112.2 | 28.0 | 124.7 | 72.8 | -89.3% | 7.51 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 7422 | 1453 | 2400 | 792.3 | 1170 | 4064 | -67.7% | 4.29 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 797.8 | 145.3 | 240.3 | 50.3 | 108.8 | 63.8 | -69.9% | 5.13 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 2036 | 108.1 | 133.0 | 36.3 | 80.7 | 506.7 | -93.5% | 23.6 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 10294 | 797.9 | 2526 | 518.6 | 672.9 | 4226 | -75.5% | 11.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1867 | 83.6 | 336.3 | 78.8 | 81.2 | 414.8 | -82.0% | 18.8 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p99_us | 10076 | 2921 | 3788 | 1268 | 2251 | 439.7 | -62.4% | 2.79 | no difference |
| P+ vs H2 | write_p50_us | 1054 | 90.6 | 1485 | 62.9 | 78.0 | 246.4 | +41.0% | 5.53 | **P+ better** |
| P+ vs H2 | write_p99_us | 3303 | 373.3 | 5029 | 546.5 | 468.0 | 619.2 | +52.3% | 3.69 | **P+ better** |
| P+ vs H2 | commits_per_s | 750.4 | 27.1 | 558.5 | 42.1 | 35.4 | 109.3 | -25.6% | 5.42 | **P+ better** |
| P+ vs H2 | pss_mib | 43.3 | 0.02 | 43.5 | 0.03 | 0.02 | 0.15 | +0.3% | 6.46 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | -806.4 | 171.0 | -824.2 | 181.9 | 176.5 | 4593 | +2.2% | 0.10 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 219.0 | 30.8 | 697.7 | 31.4 | 31.1 | 98.5 | +218.6% | 15.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 959.1 | 358.6 | 2417 | 526.6 | 450.5 | 1431 | +152.0% | 3.24 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 185.8 | 24.3 | 532.9 | 22.0 | 23.2 | 62.3 | +186.8% | 15.0 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 33375 | 1186 | 52331 | 1956 | 1618 | 4060 | +56.8% | 11.7 | **N better** |
| N vs P+ | q6_p50_us | 53031 | 2681 | 52920 | 4035 | 3426 | 2705 | -0.2% | 0.03 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 287.1 | 36.1 | 1010 | 103.0 | 77.2 | 371.3 | +251.8% | 9.36 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q1_p99_us | 1199 | 234.7 | 5751 | 780.1 | 576.1 | 205.9 | +379.6% | 7.90 | **N better** |
| N vs P+ | c2_q2_p50_us | 264.5 | 70.0 | 829.0 | 85.4 | 78.1 | 336.2 | +213.4% | 7.23 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 450.0 | 64.5 | 2083 | 145.3 | 112.4 | 595.8 | +362.8% | 14.5 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q1_p99_us | 1655 | 203.8 | 10231 | 1063 | 765.1 | 3217 | +518.1% | 11.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 358.3 | 51.3 | 1938 | 176.4 | 129.9 | 383.2 | +440.8% | 12.2 | **N better** |
| N vs P+ | c4_q2_p99_us | 1874 | 355.4 | 11273 | 1031 | 770.9 | 10255 | +501.6% | 12.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 589.1 | 51.1 | 1090 | 100.7 | 79.9 | 186.9 | +85.0% | 6.27 | **N better** |
| N vs P+ | write_p99_us | 1781 | 879.9 | 3486 | 537.1 | 728.9 | 1569 | +95.8% | 2.34 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1379 | 110.7 | 767.6 | 48.5 | 85.4 | 692.2 | -44.4% | 7.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 86.5 | 2.69 | 87.0 | 0.05 | 1.90 | 4.89 | +0.5% | 0.24 | BELOW FLOOR |
| N vs P+ | pss_growth_bytes_per_key_read | 20717 | 3440 | -5796 | 1352 | 2614 | 38686 | -128.0% | 10.1 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 219.0 | 30.8 | 106.5 | 12.9 | 23.6 | 84.6 | -51.4% | 4.76 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q1_p99_us | 959.1 | 358.6 | 586.7 | 155.1 | 276.3 | 632.1 | -38.8% | 1.35 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 185.8 | 24.3 | 209.5 | 21.5 | 22.9 | 57.9 | +12.8% | 1.03 | BELOW FLOOR |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 33375 | 1186 | 174.4 | 50.3 | 839.7 | 3580 | -99.5% | 39.5 | **P better** |
| N vs P | q6_p50_us | 53031 | 2681 | 3977 | 702.1 | 1960 | 1657 | -92.5% | 25.0 | **P better** |
| N vs P | c2_q1_p50_us | 287.1 | 36.1 | 144.4 | 56.9 | 47.7 | 106.1 | -49.7% | 2.99 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 1199 | 234.7 | 2289 | 412.6 | 335.7 | 1998 | +90.9% | 3.25 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 264.5 | 70.0 | 285.0 | 55.8 | 63.3 | 96.1 | +7.8% | 0.32 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 450.0 | 64.5 | 169.6 | 18.2 | 47.4 | 136.2 | -62.3% | 5.91 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1655 | 203.8 | 2839 | 663.3 | 490.7 | 508.1 | +71.5% | 2.41 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 358.3 | 51.3 | 334.8 | 48.1 | 49.7 | 85.7 | -6.6% | 0.47 | BELOW FLOOR |
| N vs P | c4_q2_p99_us | 1874 | 355.4 | 3705 | 476.1 | 420.1 | 1972 | +97.7% | 4.36 | BELOW FLOOR |
| N vs P | write_p50_us | 589.1 | 51.1 | 151504 | 3437 | 2430 | 11895 | +25618.2% | 62.1 | **N better** |
| N vs P | write_p99_us | 1781 | 879.9 | 199504 | 7221 | 5144 | 47242 | +11103.6% | 38.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1379 | 110.7 | 6.34 | 0.15 | 78.2 | 689.6 | -99.5% | 17.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_mib | 86.5 | 2.69 | 87.2 | 0.01 | 1.90 | 4.89 | +0.8% | 0.35 | BELOW FLOOR |
| N vs P | pss_growth_bytes_per_key_read | 20717 | 3440 | -2922 | 682.6 | 2480 | 37242 | -114.1% | 9.53 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 697.7 | 31.4 | 106.5 | 12.9 | 24.0 | 51.7 | -84.7% | 24.7 | **P better** |
| P+ vs P | q1_p99_us | 2417 | 526.6 | 586.7 | 155.1 | 388.2 | 1378 | -75.7% | 4.72 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 532.9 | 22.0 | 209.5 | 21.5 | 21.7 | 81.7 | -60.7% | 14.9 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 354.4 | 21.4 | 208.7 | 42.9 | 33.9 | 164.0 | -41.1% | 4.30 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q4_p50_us | 14451 | 300.0 | 839.7 | 77.9 | 219.2 | 104.5 | -94.2% | 62.1 | **P better** |
| P+ vs P | q5_p50_us | 52331 | 1956 | 174.4 | 50.3 | 1384 | 1914 | -99.7% | 37.7 | **P better** |
| P+ vs P | q6_p50_us | 52920 | 4035 | 3977 | 702.1 | 2896 | 2385 | -92.5% | 16.9 | **P better** |
| P+ vs P | c2_q1_p50_us | 1010 | 103.0 | 144.4 | 56.9 | 83.3 | 376.5 | -85.7% | 10.4 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q1_p99_us | 5751 | 780.1 | 2289 | 412.6 | 624.0 | 2007 | -60.2% | 5.55 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 829.0 | 85.4 | 285.0 | 55.8 | 72.2 | 341.5 | -65.6% | 7.54 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p50_us | 2083 | 145.3 | 169.6 | 18.2 | 103.5 | 593.4 | -91.9% | 18.5 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p99_us | 10231 | 1063 | 2839 | 663.3 | 885.7 | 3177 | -72.2% | 8.35 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1938 | 176.4 | 334.8 | 48.1 | 129.3 | 392.6 | -82.7% | 12.4 | **P better** |
| P+ vs P | c4_q2_p99_us | 11273 | 1031 | 3705 | 476.1 | 802.8 | 10419 | -67.1% | 9.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 1090 | 100.7 | 151504 | 3437 | 2431 | 11896 | +13798.1% | 61.9 | **P+ better** |
| P+ vs P | write_p99_us | 3486 | 537.1 | 199504 | 7221 | 5120 | 47216 | +5623.0% | 38.3 | **P+ better** |
| P+ vs P | commits_per_s | 767.6 | 48.5 | 6.34 | 0.15 | 34.3 | 59.3 | -99.2% | 22.2 | **P+ better** |
| P+ vs P | pss_mib | 87.0 | 0.05 | 87.2 | 0.01 | 0.03 | 0.01 | +0.2% | 6.02 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -5796 | 1352 | -2922 | 682.6 | 1071 | 13589 | -49.6% | 2.68 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 106.5 | 12.9 | 106.5 | 20.7 | 17.2 | 29.5 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q1_p99_us | 586.7 | 155.1 | 636.1 | 114.0 | 136.1 | 360.8 | +8.4% | 0.36 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 209.5 | 21.5 | 191.9 | 12.2 | 17.5 | 70.1 | -8.4% | 1.01 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 208.7 | 42.9 | 204.0 | 41.7 | 42.3 | 71.3 | -2.2% | 0.11 | BELOW FLOOR |
| P vs M | q4_p50_us | 839.7 | 77.9 | 873.3 | 62.5 | 70.6 | 108.8 | +4.0% | 0.48 | BELOW FLOOR |
| P vs M | q5_p50_us | 174.4 | 50.3 | 159.8 | 27.4 | 40.5 | 37.7 | -8.4% | 0.36 | BELOW FLOOR |
| P vs M | q6_p50_us | 3977 | 702.1 | 4624 | 454.9 | 591.5 | 1353 | +16.3% | 1.09 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p50_us | 144.4 | 56.9 | 100.1 | 44.9 | 51.3 | 87.6 | -30.7% | 0.86 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 2289 | 412.6 | 1695 | 457.7 | 435.7 | 2018 | -25.9% | 1.36 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 285.0 | 55.8 | 235.7 | 78.0 | 67.8 | 82.0 | -17.3% | 0.73 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 169.6 | 18.2 | 160.1 | 9.09 | 14.4 | 90.2 | -5.6% | 0.66 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2839 | 663.3 | 3149 | 774.4 | 721.0 | 1252 | +10.9% | 0.43 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 334.8 | 48.1 | 312.5 | 61.7 | 55.3 | 85.6 | -6.6% | 0.40 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3705 | 476.1 | 4606 | 1541 | 1141 | 2473 | +24.3% | 0.79 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 151504 | 3437 | 152806 | 4318 | 3902 | 12466 | +0.9% | 0.33 | BELOW FLOOR |
| P vs M | write_p99_us | 199504 | 7221 | 195399 | 7352 | 7287 | 49773 | -2.1% | 0.56 | BELOW FLOOR |
| P vs M | commits_per_s | 6.34 | 0.15 | 6.42 | 0.13 | 0.14 | 0.60 | +1.2% | 0.54 | BELOW FLOOR |
| P vs M | pss_mib | 87.2 | 0.01 | 87.2 | 0.01 | 0.01 | 0.06 | -0.0% | 1.71 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2922 | 682.6 | -1940 | 376.8 | 551.4 | 7436 | -33.6% | 1.78 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 219.0 | 30.8 | 453.3 | 52.5 | 43.1 | 88.1 | +107.0% | 5.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q1_p99_us | 959.1 | 358.6 | 2064 | 413.5 | 387.1 | 657.6 | +115.2% | 2.86 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 185.8 | 24.3 | 416.2 | 49.5 | 39.0 | 114.6 | +124.0% | 5.91 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 33375 | 1186 | 56403 | 2450 | 1925 | 3722 | +69.0% | 12.0 | **N better** |
| N vs H3 | q6_p50_us | 53031 | 2681 | 60755 | 3187 | 2945 | 1569 | +14.6% | 2.62 | no difference |
| N vs H3 | c2_q1_p50_us | 287.1 | 36.1 | 430.6 | 45.3 | 41.0 | 68.4 | +50.0% | 3.50 | **N better** |
| N vs H3 | c2_q1_p99_us | 1199 | 234.7 | 2321 | 742.4 | 550.5 | 133.1 | +93.6% | 2.04 | no difference |
| N vs H3 | c2_q2_p50_us | 264.5 | 70.0 | 397.3 | 43.6 | 58.3 | 221.4 | +50.2% | 2.28 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 450.0 | 64.5 | 706.3 | 41.4 | 54.2 | 155.2 | +56.9% | 4.72 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q1_p99_us | 1655 | 203.8 | 3114 | 256.9 | 231.9 | 3253 | +88.1% | 6.29 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 358.3 | 51.3 | 773.6 | 29.3 | 41.8 | 253.6 | +115.9% | 9.95 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p99_us | 1874 | 355.4 | 3362 | 965.6 | 727.5 | 2205 | +79.4% | 2.05 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p50_us | 589.1 | 51.1 | 1566 | 87.4 | 71.6 | 139.9 | +165.9% | 13.7 | **N better** |
| N vs H3 | write_p99_us | 1781 | 879.9 | 5153 | 638.6 | 768.8 | 1581 | +189.4% | 4.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1379 | 110.7 | 540.4 | 29.6 | 81.0 | 689.7 | -60.8% | 10.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 86.5 | 2.69 | 106.8 | 1.56 | 2.20 | 4.92 | +23.4% | 9.21 | **N better** |
| N vs H3 | pss_growth_bytes_per_key_read | 20717 | 3440 | 4881 | 526.9 | 2461 | 37559 | -76.4% | 6.43 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 697.7 | 31.4 | 453.3 | 52.5 | 43.3 | 57.2 | -35.0% | 5.65 | **H3 better** |
| P+ vs H3 | q1_p99_us | 2417 | 526.6 | 2064 | 413.5 | 473.4 | 1390 | -14.6% | 0.75 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 532.9 | 22.0 | 416.2 | 49.5 | 38.3 | 128.3 | -21.9% | 3.05 | BELOW FLOOR |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 354.4 | 21.4 | 435.4 | 46.9 | 36.4 | 153.8 | +22.8% | 2.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | 14451 | 300.0 | 14997 | 1099 | 805.2 | 1710 | +3.8% | 0.68 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 52331 | 1956 | 56403 | 2450 | 2217 | 2167 | +7.8% | 1.84 | no difference |
| P+ vs H3 | q6_p50_us | 52920 | 4035 | 60755 | 3187 | 3636 | 2325 | +14.8% | 2.16 | no difference |
| P+ vs H3 | c2_q1_p50_us | 1010 | 103.0 | 430.6 | 45.3 | 79.6 | 367.7 | -57.4% | 7.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c2_q1_p99_us | 5751 | 780.1 | 2321 | 742.4 | 761.5 | 237.0 | -59.6% | 4.50 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 829.0 | 85.4 | 397.3 | 43.6 | 67.8 | 395.5 | -52.1% | 6.36 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q1_p50_us | 2083 | 145.3 | 706.3 | 41.4 | 106.8 | 598.0 | -66.1% | 12.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q1_p99_us | 10231 | 1063 | 3114 | 256.9 | 773.0 | 4518 | -69.6% | 9.21 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1938 | 176.4 | 773.6 | 29.3 | 126.5 | 459.5 | -60.1% | 9.21 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 11273 | 1031 | 3362 | 965.6 | 998.7 | 10465 | -70.2% | 7.92 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1090 | 100.7 | 1566 | 87.4 | 94.3 | 209.5 | +43.7% | 5.05 | **P+ better** |
| P+ vs H3 | write_p99_us | 3486 | 537.1 | 5153 | 638.6 | 590.0 | 227.5 | +47.8% | 2.82 | no difference |
| P+ vs H3 | commits_per_s | 767.6 | 48.5 | 540.4 | 29.6 | 40.2 | 59.5 | -29.6% | 5.65 | **P+ better** |
| P+ vs H3 | pss_mib | 87.0 | 0.05 | 106.8 | 1.56 | 1.10 | 0.57 | +22.8% | 17.9 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | -5796 | 1352 | 4881 | 526.9 | 1026 | 14434 | -184.2% | 10.4 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs T | q1_p50_us | 219.0 | 30.8 | 1468 | 35.4 | 33.2 | 86.3 | +570.4% | 37.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | q1_p99_us | 959.1 | 358.6 | 4866 | 1404 | 1024 | 1600 | +407.4% | 3.81 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| N vs T | q2_p50_us | 185.8 | 24.3 | 1475 | 47.0 | 37.4 | 208.1 | +694.2% | 34.5 | **N better** |
| N vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs T | c2_q1_p50_us | 287.1 | 36.1 | 2061 | 143.1 | 104.4 | 238.3 | +617.8% | 17.0 | **N better** |
| N vs T | c2_q1_p99_us | 1199 | 234.7 | 4718 | 835.6 | 613.7 | 644.8 | +293.5% | 5.73 | **N better** |
| N vs T | c2_q2_p50_us | 264.5 | 70.0 | 2092 | 79.7 | 75.0 | 108.1 | +690.9% | 24.4 | **N better** |
| N vs T | c4_q1_p50_us | 450.0 | 64.5 | 2638 | 143.3 | 111.1 | 666.4 | +486.2% | 19.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q1_p99_us | 1655 | 203.8 | 6931 | 1395 | 997.1 | 2411 | +318.8% | 5.29 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | c4_q2_p50_us | 358.3 | 51.3 | 2620 | 236.5 | 171.1 | 569.5 | +631.3% | 13.2 | **N better** |
| N vs T | c4_q2_p99_us | 1874 | 355.4 | 7740 | 3077 | 2190 | 4314 | +313.1% | 2.68 | REFUSED (warm-up MAD above 15% of the median on T) |
| N vs T | write_p50_us | 589.1 | 51.1 | 1344 | 29.5 | 41.7 | 75.5 | +128.1% | 18.1 | **N better** |
| N vs T | write_p99_us | 1781 | 879.9 | 2454 | 487.1 | 711.2 | 1648 | +37.8% | 0.95 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | commits_per_s | 1379 | 110.7 | 663.5 | 77.5 | 95.5 | 691.5 | -51.9% | 7.49 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs T | pss_mib | 86.5 | 2.69 | 2580 | 0.15 | 1.90 | 4.89 | +2882.2% | 1309 | **N better** |
| N vs T | pss_growth_bytes_per_key_read | 20717 | 3440 | -253.1 | 93.7 | 2434 | 36749 | -101.2% | 8.62 | REFUSED (warm-up MAD above 15% of the median on N and T) |
| P+ vs T | q1_p50_us | 697.7 | 31.4 | 1468 | 35.4 | 33.5 | 54.3 | +110.5% | 23.0 | **P+ better** |
| P+ vs T | q1_p99_us | 2417 | 526.6 | 4866 | 1404 | 1060 | 2015 | +101.3% | 2.31 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | q2_p50_us | 532.9 | 22.0 | 1475 | 47.0 | 36.7 | 216.0 | +176.9% | 25.7 | **P+ better** |
| P+ vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| P+ vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs T | c2_q1_p50_us | 1010 | 103.0 | 2061 | 143.1 | 124.7 | 432.8 | +104.0% | 8.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c2_q1_p99_us | 5751 | 780.1 | 4718 | 835.6 | 808.3 | 673.9 | -18.0% | 1.28 | no difference |
| P+ vs T | c2_q2_p50_us | 829.0 | 85.4 | 2092 | 79.7 | 82.6 | 345.1 | +152.3% | 15.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q1_p50_us | 2083 | 145.3 | 2638 | 143.3 | 144.3 | 881.8 | +26.6% | 3.85 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q1_p99_us | 10231 | 1063 | 6931 | 1395 | 1240 | 3956 | -32.3% | 2.66 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs T | c4_q2_p50_us | 1938 | 176.4 | 2620 | 236.5 | 208.6 | 686.4 | +35.2% | 3.27 | BELOW FLOOR |
| P+ vs T | c4_q2_p99_us | 11273 | 1031 | 7740 | 3077 | 2295 | 11103 | -31.3% | 1.54 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| P+ vs T | write_p50_us | 1090 | 100.7 | 1344 | 29.5 | 74.2 | 173.2 | +23.3% | 3.42 | **P+ better** |
| P+ vs T | write_p99_us | 3486 | 537.1 | 2454 | 487.1 | 512.7 | 518.7 | -29.6% | 2.01 | no difference |
| P+ vs T | commits_per_s | 767.6 | 48.5 | 663.5 | 77.5 | 64.7 | 78.1 | -13.6% | 1.61 | no difference |
| P+ vs T | pss_mib | 87.0 | 0.05 | 2580 | 0.15 | 0.11 | 0.22 | +2866.5% | 21881 | **P+ better** |
| P+ vs T | pss_growth_bytes_per_key_read | -5796 | 1352 | -253.1 | 93.7 | 958.6 | 12171 | -95.6% | 5.78 | REFUSED (warm-up MAD above 15% of the median on P+ and T) |
| H3 vs T | q1_p50_us | 453.3 | 52.5 | 1468 | 35.4 | 44.8 | 31.8 | +223.9% | 22.6 | **H3 better** |
| H3 vs T | q1_p99_us | 2064 | 413.5 | 4866 | 1404 | 1035 | 1564 | +135.8% | 2.71 | REFUSED (warm-up MAD above 15% of the median on T) |
| H3 vs T | q2_p50_us | 416.2 | 49.5 | 1475 | 47.0 | 48.3 | 236.4 | +254.5% | 21.9 | **H3 better** |
| H3 vs T | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| H3 vs T | q5_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| H3 vs T | c2_q1_p50_us | 430.6 | 45.3 | 2061 | 143.1 | 106.1 | 232.6 | +378.6% | 15.4 | **H3 better** |
| H3 vs T | c2_q1_p99_us | 2321 | 742.4 | 4718 | 835.6 | 790.3 | 655.3 | +103.2% | 3.03 | **H3 better** |
| H3 vs T | c2_q2_p50_us | 397.3 | 43.6 | 2092 | 79.7 | 64.2 | 234.6 | +426.5% | 26.4 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q1_p50_us | 706.3 | 41.4 | 2638 | 143.3 | 105.5 | 668.4 | +273.5% | 18.3 | **H3 better** |
| H3 vs T | c4_q1_p99_us | 3114 | 256.9 | 6931 | 1395 | 1003 | 3985 | +122.6% | 3.80 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p50_us | 773.6 | 29.3 | 2620 | 236.5 | 168.5 | 623.4 | +238.7% | 11.0 | REFUSED (warm-up MAD above 15% of the median on H3) |
| H3 vs T | c4_q2_p99_us | 3362 | 965.6 | 7740 | 3077 | 2281 | 4792 | +130.2% | 1.92 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| H3 vs T | write_p50_us | 1566 | 87.4 | 1344 | 29.5 | 65.2 | 121.0 | -14.2% | 3.41 | **T better** |
| H3 vs T | write_p99_us | 5153 | 638.6 | 2454 | 487.1 | 567.9 | 555.5 | -52.4% | 4.75 | **T better** |
| H3 vs T | commits_per_s | 540.4 | 29.6 | 663.5 | 77.5 | 58.7 | 51.1 | +22.8% | 2.10 | no difference |
| H3 vs T | pss_mib | 106.8 | 1.56 | 2580 | 0.15 | 1.11 | 0.61 | +2316.4% | 2229 | **H3 better** |
| H3 vs T | pss_growth_bytes_per_key_read | 4881 | 526.9 | -253.1 | 93.7 | 378.5 | 7884 | -105.2% | 13.6 | REFUSED (warm-up MAD above 15% of the median on H3 and T) |
| N vs H1 | q1_p50_us | 219.0 | 30.8 | 1097 | 39.9 | 35.6 | 195.8 | +400.8% | 24.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q1_p99_us | 959.1 | 358.6 | 6031 | 1233 | 908.1 | 532.6 | +528.9% | 5.59 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | q2_p50_us | 185.8 | 24.3 | 909.1 | 121.8 | 87.8 | 81.9 | +389.3% | 8.24 | **N better** |
| N vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H1 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H1 | q5_p50_us | 33375 | 1186 | 741.3 | 196.4 | 850.4 | 50595 | -97.8% | 38.4 | BELOW FLOOR |
| N vs H1 | q6_p50_us | 53031 | 2681 | 8233 | 456.9 | 1923 | 2474 | -84.5% | 23.3 | **H1 better** |
| N vs H1 | c2_q1_p50_us | 287.1 | 36.1 | 1001 | 58.7 | 48.8 | 64.2 | +248.7% | 14.6 | **N better** |
| N vs H1 | c2_q1_p99_us | 1199 | 234.7 | 6625 | 1317 | 946.1 | 2761 | +452.5% | 5.73 | REFUSED (warm-up MAD above 15% of the median on H1) |
| N vs H1 | c2_q2_p50_us | 264.5 | 70.0 | 988.0 | 64.3 | 67.2 | 62.5 | +273.5% | 10.8 | **N better** |
| N vs H1 | c4_q1_p50_us | 450.0 | 64.5 | 994.5 | 70.8 | 67.7 | 132.8 | +121.0% | 8.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q1_p99_us | 1655 | 203.8 | 5583 | 930.0 | 673.2 | 647.9 | +237.3% | 5.83 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | c4_q2_p50_us | 358.3 | 51.3 | 937.8 | 50.2 | 50.7 | 10.2 | +161.7% | 11.4 | **N better** |
| N vs H1 | c4_q2_p99_us | 1874 | 355.4 | 4974 | 1039 | 776.6 | 665.3 | +165.4% | 3.99 | **N better** |
| N vs H1 | write_p50_us | 589.1 | 51.1 | 1703 | 82.5 | 68.6 | 131.2 | +189.1% | 16.2 | **N better** |
| N vs H1 | write_p99_us | 1781 | 879.9 | 4718 | 159.1 | 632.3 | 1922 | +165.0% | 4.65 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | commits_per_s | 1379 | 110.7 | 507.5 | 46.5 | 84.9 | 689.8 | -63.2% | 10.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H1 | pss_mib | 86.5 | 2.69 | 426.9 | 4.94 | 3.98 | 6.49 | +393.5% | 85.6 | **N better** |
| N vs H1 | pss_growth_bytes_per_key_read | 20717 | 3440 | 14.0 | 1653 | 2699 | 76114 | -99.9% | 7.67 | REFUSED (warm-up MAD above 15% of the median on N and H1) |
| P+ vs H1 | q1_p50_us | 697.7 | 31.4 | 1097 | 39.9 | 35.9 | 183.9 | +57.2% | 11.1 | **P+ better** |
| P+ vs H1 | q1_p99_us | 2417 | 526.6 | 6031 | 1233 | 948.2 | 1335 | +149.5% | 3.81 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q2_p50_us | 532.9 | 22.0 | 909.1 | 121.8 | 87.5 | 100.2 | +70.6% | 4.30 | **P+ better** |
| P+ vs H1 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H1 | q3_p50_us | 354.4 | 21.4 | 1047 | 146.1 | 104.4 | 237.9 | +195.5% | 6.64 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | q4_p50_us | 14451 | 300.0 | 30579 | 1340 | 970.8 | 10960 | +111.6% | 16.6 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | q5_p50_us | 52331 | 1956 | 741.3 | 196.4 | 1390 | 50504 | -98.6% | 37.1 | **H1 better** |
| P+ vs H1 | q6_p50_us | 52920 | 4035 | 8233 | 456.9 | 2871 | 3011 | -84.4% | 15.6 | **H1 better** |
| P+ vs H1 | c2_q1_p50_us | 1010 | 103.0 | 1001 | 58.7 | 83.9 | 366.9 | -0.9% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c2_q1_p99_us | 5751 | 780.1 | 6625 | 1317 | 1083 | 2768 | +15.2% | 0.81 | REFUSED (warm-up MAD above 15% of the median on H1) |
| P+ vs H1 | c2_q2_p50_us | 829.0 | 85.4 | 988.0 | 64.3 | 75.6 | 333.6 | +19.2% | 2.10 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q1_p50_us | 2083 | 145.3 | 994.5 | 70.8 | 114.3 | 592.6 | -52.3% | 9.53 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q1_p99_us | 10231 | 1063 | 5583 | 930.0 | 998.5 | 3202 | -45.4% | 4.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | c4_q2_p50_us | 1938 | 176.4 | 937.8 | 50.2 | 129.7 | 383.3 | -51.6% | 7.71 | **H1 better** |
| P+ vs H1 | c4_q2_p99_us | 11273 | 1031 | 4974 | 1039 | 1035 | 10252 | -55.9% | 6.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H1 | write_p50_us | 1090 | 100.7 | 1703 | 82.5 | 92.0 | 203.7 | +56.3% | 6.66 | **P+ better** |
| P+ vs H1 | write_p99_us | 3486 | 537.1 | 4718 | 159.1 | 396.1 | 1117 | +35.3% | 3.11 | **P+ better** |
| P+ vs H1 | commits_per_s | 767.6 | 48.5 | 507.5 | 46.5 | 47.5 | 61.0 | -33.9% | 5.47 | **P+ better** |
| P+ vs H1 | pss_mib | 87.0 | 0.05 | 426.9 | 4.94 | 3.49 | 4.28 | +390.9% | 97.4 | **P+ better** |
| P+ vs H1 | pss_growth_bytes_per_key_read | -5796 | 1352 | 14.0 | 1653 | 1510 | 67757 | -100.2% | 3.85 | REFUSED (warm-up MAD above 15% of the median on P+ and H1) |
| N vs H2 | q1_p50_us | 219.0 | 30.8 | 122.7 | 14.1 | 24.0 | 85.4 | -44.0% | 4.02 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q1_p99_us | 959.1 | 358.6 | 727.4 | 201.6 | 290.9 | 525.8 | -24.2% | 0.80 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 185.8 | 24.3 | 208.8 | 25.6 | 24.9 | 18.8 | +12.4% | 0.92 | no difference |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 33375 | 1186 | 205.0 | 29.5 | 839.2 | 3580 | -99.4% | 39.5 | **H2 better** |
| N vs H2 | q6_p50_us | 53031 | 2681 | 3537 | 259.2 | 1905 | 2174 | -93.3% | 26.0 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q1_p50_us | 287.1 | 36.1 | 128.1 | 35.3 | 35.7 | 65.0 | -55.4% | 4.45 | **H2 better** |
| N vs H2 | c2_q1_p99_us | 1199 | 234.7 | 1708 | 393.2 | 323.8 | 134.1 | +42.4% | 1.57 | no difference |
| N vs H2 | c2_q2_p50_us | 264.5 | 70.0 | 262.3 | 51.8 | 61.6 | 56.5 | -0.8% | 0.04 | BELOW FLOOR |
| N vs H2 | c4_q1_p50_us | 450.0 | 64.5 | 207.8 | 36.9 | 52.6 | 105.9 | -53.8% | 4.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p99_us | 1655 | 203.8 | 3072 | 423.1 | 332.1 | 668.6 | +85.6% | 4.27 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 358.3 | 51.3 | 441.4 | 69.2 | 60.9 | 8.74 | +23.2% | 1.36 | no difference |
| N vs H2 | c4_q2_p99_us | 1874 | 355.4 | 4399 | 708.2 | 560.3 | 613.9 | +134.7% | 4.51 | **N better** |
| N vs H2 | write_p50_us | 589.1 | 51.1 | 1547 | 155.6 | 115.8 | 153.2 | +162.7% | 8.28 | **N better** |
| N vs H2 | write_p99_us | 1781 | 879.9 | 4840 | 316.6 | 661.2 | 2051 | +171.8% | 4.63 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1379 | 110.7 | 550.7 | 40.2 | 83.2 | 695.1 | -60.1% | 9.96 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_mib | 86.5 | 2.69 | 86.4 | 0.02 | 1.90 | 4.89 | -0.1% | 0.05 | BELOW FLOOR |
| N vs H2 | pss_growth_bytes_per_key_read | 20717 | 3440 | -1079 | 217.6 | 2438 | 36820 | -105.2% | 8.94 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 697.7 | 31.4 | 122.7 | 14.1 | 24.3 | 52.9 | -82.4% | 23.6 | **H2 better** |
| P+ vs H2 | q1_p99_us | 2417 | 526.6 | 727.4 | 201.6 | 398.7 | 1333 | -69.9% | 4.24 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 532.9 | 22.0 | 208.8 | 25.6 | 23.8 | 60.7 | -60.8% | 13.6 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 354.4 | 21.4 | 232.6 | 16.8 | 19.2 | 165.4 | -34.4% | 6.33 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q4_p50_us | 14451 | 300.0 | 907.2 | 48.8 | 214.9 | 118.6 | -93.7% | 63.0 | **H2 better** |
| P+ vs H2 | q5_p50_us | 52331 | 1956 | 205.0 | 29.5 | 1383 | 1914 | -99.6% | 37.7 | **H2 better** |
| P+ vs H2 | q6_p50_us | 52920 | 4035 | 3537 | 259.2 | 2859 | 2769 | -93.3% | 17.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q1_p50_us | 1010 | 103.0 | 128.1 | 35.3 | 77.0 | 367.1 | -87.3% | 11.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q1_p99_us | 5751 | 780.1 | 1708 | 393.2 | 617.7 | 237.5 | -70.3% | 6.54 | **H2 better** |
| P+ vs H2 | c2_q2_p50_us | 829.0 | 85.4 | 262.3 | 51.8 | 70.6 | 332.6 | -68.4% | 8.02 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q1_p50_us | 2083 | 145.3 | 207.8 | 36.9 | 106.0 | 587.1 | -90.0% | 17.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q1_p99_us | 10231 | 1063 | 3072 | 423.1 | 808.8 | 3206 | -70.0% | 8.85 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1938 | 176.4 | 441.4 | 69.2 | 134.0 | 383.2 | -77.2% | 11.2 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 11273 | 1031 | 4399 | 708.2 | 884.3 | 10249 | -61.0% | 7.77 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 1090 | 100.7 | 1547 | 155.6 | 131.0 | 218.6 | +42.0% | 3.49 | **P+ better** |
| P+ vs H2 | write_p99_us | 3486 | 537.1 | 4840 | 316.6 | 440.8 | 1326 | +38.8% | 3.07 | **P+ better** |
| P+ vs H2 | commits_per_s | 767.6 | 48.5 | 550.7 | 40.2 | 44.5 | 105.6 | -28.3% | 4.87 | **P+ better** |
| P+ vs H2 | pss_mib | 87.0 | 0.05 | 86.4 | 0.02 | 0.03 | 0.11 | -0.6% | 16.1 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | -5796 | 1352 | -1079 | 217.6 | 968.7 | 12385 | -81.4% | 4.87 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 137.6 | 13.1 | 566.7 | 18.5 | 16.0 | 25.0 | +311.8% | 26.8 | **N better** |
| N vs P+ | q1_p99_us | 980.5 | 474.2 | 3908 | 850.9 | 688.8 | 1803 | +298.6% | 4.25 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 161.6 | 18.3 | 493.9 | 26.4 | 22.7 | 40.2 | +205.6% | 14.6 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 835748 | 50995 | 1118412 | 21766 | 39206 | 77660 | +33.8% | 7.21 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 93.3 | 23.1 | 527.6 | 21.2 | 22.2 | 99.5 | +465.3% | 19.6 | **N better** |
| N vs P+ | c2_q1_p99_us | 296.3 | 65.0 | 5016 | 648.0 | 460.5 | 1572 | +1592.6% | 10.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 115.9 | 22.9 | 498.7 | 24.9 | 23.9 | 122.1 | +330.3% | 16.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 137.3 | 24.7 | 1535 | 130.4 | 93.8 | 78.5 | +1017.8% | 14.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 1177 | 430.4 | 8414 | 585.2 | 513.7 | 2058 | +614.9% | 14.1 | **N better** |
| N vs P+ | c4_q2_p50_us | 154.9 | 21.6 | 1385 | 90.7 | 65.9 | 295.9 | +793.9% | 18.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1784 | 1034 | 5076 | 2029 | 1610 | 9316 | +184.5% | 2.04 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 645.2 | 48.7 | 1080 | 75.0 | 63.2 | 203.6 | +67.4% | 6.88 | **N better** |
| N vs P+ | write_p99_us | 1945 | 682.6 | 3652 | 234.0 | 510.2 | 3259 | +87.8% | 3.35 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1253 | 191.8 | 731.1 | 48.2 | 139.8 | 223.8 | -41.7% | 3.73 | **N better** |
| N vs P+ | pss_mib | 852.6 | 14.2 | 493.9 | 0.60 | 10.0 | 0.58 | -42.1% | 35.7 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 157366 | 36865 | 52465 | 12610 | 27551 | 338653 | -66.7% | 3.81 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 137.6 | 13.1 | 148.6 | 2.00 | 9.36 | 39.2 | +7.9% | 1.17 | BELOW FLOOR |
| N vs P | q1_p99_us | 980.5 | 474.2 | 1062 | 491.4 | 482.9 | 201.2 | +8.3% | 0.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 161.6 | 18.3 | 234.8 | 11.4 | 15.2 | 16.5 | +45.3% | 4.80 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 835748 | 50995 | 198.9 | 15.4 | 36059 | 64405 | -100.0% | 23.2 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 93.3 | 23.1 | 61.5 | 2.26 | 16.4 | 50.1 | -34.1% | 1.94 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 296.3 | 65.0 | 1279 | 112.4 | 91.8 | 440.6 | +331.6% | 10.7 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 115.9 | 22.9 | 175.8 | 23.0 | 22.9 | 122.0 | +51.7% | 2.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 137.3 | 24.7 | 150.4 | 38.9 | 32.6 | 89.7 | +9.6% | 0.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 1177 | 430.4 | 2943 | 397.3 | 414.2 | 2051 | +150.1% | 4.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 154.9 | 21.6 | 343.6 | 49.6 | 38.3 | 164.7 | +121.8% | 4.93 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 1784 | 1034 | 6460 | 2160 | 1693 | 9796 | +262.1% | 2.76 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 645.2 | 48.7 | 1583559 | 46969 | 33212 | 72794 | +245321.9% | 47.7 | **N better** |
| N vs P | write_p99_us | 1945 | 682.6 | 1788359 | 50374 | 35623 | 55495 | +91845.6% | 50.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1253 | 191.8 | 0.63 | 0.01 | 135.6 | 223.0 | -100.0% | 9.24 | **N better** |
| N vs P | pss_mib | 852.6 | 14.2 | 486.6 | 1.72 | 10.1 | 0.42 | -42.9% | 36.2 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 157366 | 36865 | 51758 | 12339 | 27489 | 341288 | -67.1% | 3.84 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 566.7 | 18.5 | 148.6 | 2.00 | 13.1 | 41.8 | -73.8% | 31.9 | **P better** |
| P+ vs P | q1_p99_us | 3908 | 850.9 | 1062 | 491.4 | 694.8 | 1795 | -72.8% | 4.10 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 493.9 | 26.4 | 234.8 | 11.4 | 20.3 | 38.9 | -52.5% | 12.8 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 294.2 | 12.0 | 285.5 | 25.7 | 20.1 | 86.6 | -3.0% | 0.43 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 85307 | 2143 | 6094 | 175.1 | 1520 | 14774 | -92.9% | 52.1 | **P better** |
| P+ vs P | q5_p50_us | 1118412 | 21766 | 198.9 | 15.4 | 15391 | 43393 | -100.0% | 72.7 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 527.6 | 21.2 | 61.5 | 2.26 | 15.1 | 86.1 | -88.3% | 30.9 | **P better** |
| P+ vs P | c2_q1_p99_us | 5016 | 648.0 | 1279 | 112.4 | 465.0 | 1621 | -74.5% | 8.04 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 498.7 | 24.9 | 175.8 | 23.0 | 24.0 | 20.9 | -64.7% | 13.5 | **P better** |
| P+ vs P | c4_q1_p50_us | 1535 | 130.4 | 150.4 | 38.9 | 96.2 | 112.2 | -90.2% | 14.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8414 | 585.2 | 2943 | 397.3 | 500.2 | 2890 | -65.0% | 10.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1385 | 90.7 | 343.6 | 49.6 | 73.1 | 282.0 | -75.2% | 14.2 | **P better** |
| P+ vs P | c4_q2_p99_us | 5076 | 2029 | 6460 | 2160 | 2096 | 6871 | +27.3% | 0.66 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 1080 | 75.0 | 1583559 | 46969 | 33212 | 72794 | +146485.9% | 47.6 | **P+ better** |
| P+ vs P | write_p99_us | 3652 | 234.0 | 1788359 | 50374 | 35620 | 55512 | +48866.3% | 50.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 731.1 | 48.2 | 0.63 | 0.01 | 34.0 | 19.4 | -99.9% | 21.5 | **P+ better** |
| P+ vs P | pss_mib | 493.9 | 0.60 | 486.6 | 1.72 | 1.29 | 0.41 | -1.5% | 5.63 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 52465 | 12610 | 51758 | 12339 | 12475 | 192909 | -1.3% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 148.6 | 2.00 | 149.7 | 3.59 | 2.91 | 36.5 | +0.8% | 0.40 | BELOW FLOOR |
| P vs M | q1_p99_us | 1062 | 491.4 | 1242 | 603.6 | 550.4 | 1030 | +16.9% | 0.33 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 234.8 | 11.4 | 229.3 | 16.3 | 14.0 | 9.97 | -2.3% | 0.39 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 285.5 | 25.7 | 279.5 | 39.7 | 33.5 | 89.2 | -2.1% | 0.18 | BELOW FLOOR |
| P vs M | q4_p50_us | 6094 | 175.1 | 6043 | 69.3 | 133.2 | 21.2 | -0.8% | 0.39 | no difference |
| P vs M | q5_p50_us | 198.9 | 15.4 | 189.3 | 14.3 | 14.9 | 70.7 | -4.8% | 0.65 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 61.5 | 2.26 | 60.0 | 2.07 | 2.17 | 35.7 | -2.5% | 0.70 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1279 | 112.4 | 1084 | 43.2 | 85.2 | 1255 | -15.3% | 2.29 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 175.8 | 23.0 | 179.6 | 25.0 | 24.0 | 49.3 | +2.1% | 0.16 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 150.4 | 38.9 | 154.1 | 26.2 | 33.2 | 86.3 | +2.4% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2943 | 397.3 | 3079 | 604.6 | 511.6 | 2040 | +4.6% | 0.27 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 343.6 | 49.6 | 284.0 | 21.5 | 38.2 | 131.5 | -17.3% | 1.56 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 6460 | 2160 | 5096 | 1613 | 1906 | 20041 | -21.1% | 0.72 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 1583559 | 46969 | 1590893 | 16920 | 35301 | 75965 | +0.5% | 0.21 | BELOW FLOOR |
| P vs M | write_p99_us | 1788359 | 50374 | 1834694 | 41171 | 46003 | 63603 | +2.6% | 1.01 | BELOW FLOOR |
| P vs M | commits_per_s | 0.63 | 0.01 | 0.62 | 0.01 | 0.01 | 0.05 | -0.5% | 0.31 | BELOW FLOOR |
| P vs M | pss_mib | 486.6 | 1.72 | 487.5 | 1.05 | 1.43 | 2.95 | +0.2% | 0.60 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 51758 | 12339 | 53989 | 13062 | 12706 | 198522 | +4.3% | 0.18 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 137.6 | 13.1 | 359.5 | 36.8 | 27.6 | 15.0 | +161.3% | 8.03 | **N better** |
| N vs H3 | q1_p99_us | 980.5 | 474.2 | 2933 | 877.1 | 705.0 | 219.6 | +199.2% | 2.77 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | q2_p50_us | 161.6 | 18.3 | 338.2 | 45.3 | 34.5 | 52.4 | +109.3% | 5.11 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 835748 | 50995 | 1145841 | 33503 | 43145 | 64517 | +37.1% | 7.19 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 93.3 | 23.1 | 284.9 | 49.7 | 38.7 | 80.7 | +205.2% | 4.95 | **N better** |
| N vs H3 | c2_q1_p99_us | 296.3 | 65.0 | 3448 | 1242 | 879.4 | 1707 | +1063.4% | 3.58 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 115.9 | 22.9 | 286.9 | 30.5 | 27.0 | 171.3 | +147.5% | 6.34 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p50_us | 137.3 | 24.7 | 408.0 | 91.1 | 66.7 | 70.1 | +197.2% | 4.06 | **N better** |
| N vs H3 | c4_q1_p99_us | 1177 | 430.4 | 5046 | 1406 | 1040 | 3253 | +328.7% | 3.72 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 154.9 | 21.6 | 357.5 | 61.4 | 46.0 | 134.2 | +130.8% | 4.40 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1784 | 1034 | 5046 | 1091 | 1063 | 8272 | +182.9% | 3.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 645.2 | 48.7 | 1686 | 115.4 | 88.6 | 228.5 | +161.2% | 11.7 | **N better** |
| N vs H3 | write_p99_us | 1945 | 682.6 | 5482 | 1236 | 998.4 | 2477 | +181.9% | 3.54 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | commits_per_s | 1253 | 191.8 | 512.5 | 37.5 | 138.2 | 253.0 | -59.1% | 5.36 | **N better** |
| N vs H3 | pss_mib | 852.6 | 14.2 | 554.6 | 0.95 | 10.1 | 2.12 | -35.0% | 29.6 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 157366 | 36865 | 70342 | 13905 | 27860 | 335153 | -55.3% | 3.12 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 566.7 | 18.5 | 359.5 | 36.8 | 29.1 | 20.9 | -36.6% | 7.11 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3908 | 850.9 | 2933 | 877.1 | 864.1 | 1797 | -25.0% | 1.13 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q2_p50_us | 493.9 | 26.4 | 338.2 | 45.3 | 37.0 | 63.1 | -31.5% | 4.20 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 294.2 | 12.0 | 337.3 | 20.5 | 16.8 | 172.2 | +14.7% | 2.56 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 85307 | 2143 | 90223 | 3209 | 2729 | 14779 | +5.8% | 1.80 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 1118412 | 21766 | 1145841 | 33503 | 28251 | 43559 | +2.5% | 0.97 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 527.6 | 21.2 | 284.9 | 49.7 | 38.2 | 106.8 | -46.0% | 6.36 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5016 | 648.0 | 3448 | 1242 | 990.5 | 2313 | -31.3% | 1.58 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 498.7 | 24.9 | 286.9 | 30.5 | 27.8 | 122.1 | -42.5% | 7.61 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1535 | 130.4 | 408.0 | 91.1 | 112.5 | 97.2 | -73.4% | 10.0 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 8414 | 585.2 | 5046 | 1406 | 1077 | 3838 | -40.0% | 3.13 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1385 | 90.7 | 357.5 | 61.4 | 77.4 | 265.3 | -74.2% | 13.3 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 5076 | 2029 | 5046 | 1091 | 1629 | 4436 | -0.6% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1080 | 75.0 | 1686 | 115.4 | 97.3 | 199.9 | +56.0% | 6.22 | **P+ better** |
| P+ vs H3 | write_p99_us | 3652 | 234.0 | 5482 | 1236 | 889.5 | 2835 | +50.1% | 2.06 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 731.1 | 48.2 | 512.5 | 37.5 | 43.2 | 121.1 | -29.9% | 5.06 | **P+ better** |
| P+ vs H3 | pss_mib | 493.9 | 0.60 | 554.6 | 0.95 | 0.79 | 2.11 | +12.3% | 76.4 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 52465 | 12610 | 70342 | 13905 | 13273 | 181835 | +34.1% | 1.35 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 137.6 | 13.1 | 131.1 | 13.0 | 13.0 | 14.5 | -4.7% | 0.50 | BELOW FLOOR |
| N vs H2 | q1_p99_us | 980.5 | 474.2 | 1210 | 638.3 | 562.3 | 223.6 | +23.4% | 0.41 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 161.6 | 18.3 | 253.2 | 27.9 | 23.6 | 34.3 | +56.7% | 3.89 | **N better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 835748 | 50995 | 179.6 | 21.5 | 36059 | 64405 | -100.0% | 23.2 | **H2 better** |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 93.3 | 23.1 | 131.9 | 21.2 | 22.1 | 49.9 | +41.3% | 1.74 | BELOW FLOOR |
| N vs H2 | c2_q1_p99_us | 296.3 | 65.0 | 2111 | 366.7 | 263.3 | 653.2 | +612.4% | 6.89 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q2_p50_us | 115.9 | 22.9 | 270.3 | 39.6 | 32.3 | 142.1 | +133.2% | 4.78 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q1_p50_us | 137.3 | 24.7 | 203.1 | 24.7 | 24.7 | 37.7 | +47.9% | 2.66 | no difference |
| N vs H2 | c4_q1_p99_us | 1177 | 430.4 | 3134 | 556.8 | 497.6 | 630.6 | +166.3% | 3.93 | **N better** |
| N vs H2 | c4_q2_p50_us | 154.9 | 21.6 | 355.9 | 49.7 | 38.3 | 141.6 | +129.7% | 5.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p99_us | 1784 | 1034 | 6913 | 2428 | 1866 | 8233 | +287.5% | 2.75 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | write_p50_us | 645.2 | 48.7 | 1878 | 179.3 | 131.3 | 164.1 | +191.0% | 9.38 | **N better** |
| N vs H2 | write_p99_us | 1945 | 682.6 | 5585 | 898.7 | 798.0 | 2205 | +187.1% | 4.56 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1253 | 191.8 | 430.6 | 71.1 | 144.6 | 223.4 | -65.6% | 5.69 | **N better** |
| N vs H2 | pss_mib | 852.6 | 14.2 | 489.1 | 0.32 | 10.0 | 3.16 | -42.6% | 36.2 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 157366 | 36865 | 54540 | 13379 | 27731 | 342086 | -65.3% | 3.71 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 566.7 | 18.5 | 131.1 | 13.0 | 16.0 | 20.5 | -76.9% | 27.3 | **H2 better** |
| P+ vs H2 | q1_p99_us | 3908 | 850.9 | 1210 | 638.3 | 752.1 | 1798 | -69.0% | 3.59 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 493.9 | 26.4 | 253.2 | 27.9 | 27.1 | 49.2 | -48.7% | 8.87 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 294.2 | 12.0 | 278.5 | 23.3 | 18.6 | 36.5 | -5.3% | 0.84 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 85307 | 2143 | 6678 | 238.3 | 1525 | 14775 | -92.2% | 51.6 | **H2 better** |
| P+ vs H2 | q5_p50_us | 1118412 | 21766 | 179.6 | 21.5 | 15391 | 43393 | -100.0% | 72.7 | **H2 better** |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 527.6 | 21.2 | 131.9 | 21.2 | 21.2 | 86.0 | -75.0% | 18.7 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5016 | 648.0 | 2111 | 366.7 | 526.5 | 1691 | -57.9% | 5.52 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q2_p50_us | 498.7 | 24.9 | 270.3 | 39.6 | 33.0 | 75.8 | -45.8% | 6.91 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1535 | 130.4 | 203.1 | 24.7 | 93.8 | 77.2 | -86.8% | 14.2 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 8414 | 585.2 | 3134 | 556.8 | 571.2 | 2131 | -62.8% | 9.24 | **H2 better** |
| P+ vs H2 | c4_q2_p50_us | 1385 | 90.7 | 355.9 | 49.7 | 73.1 | 269.1 | -74.3% | 14.1 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 5076 | 2029 | 6913 | 2428 | 2237 | 4363 | +36.2% | 0.82 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | write_p50_us | 1080 | 75.0 | 1878 | 179.3 | 137.4 | 121.1 | +73.8% | 5.81 | **P+ better** |
| P+ vs H2 | write_p99_us | 3652 | 234.0 | 5585 | 898.7 | 656.6 | 2601 | +52.9% | 2.94 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 731.1 | 48.2 | 430.6 | 71.1 | 60.7 | 23.6 | -41.1% | 4.95 | **P+ better** |
| P+ vs H2 | pss_mib | 493.9 | 0.60 | 489.1 | 0.32 | 0.48 | 3.15 | -1.0% | 10.00 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | 52465 | 12610 | 54540 | 13379 | 13000 | 194317 | +4.0% | 0.16 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 154.3 | 15.3 | 548.0 | 9.00 | 12.5 | 99.4 | +255.1% | 31.4 | **N better** |
| N vs P+ | q1_p99_us | 1788 | 800.9 | 4517 | 474.8 | 658.4 | 1152 | +152.6% | 4.14 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 177.6 | 19.9 | 492.9 | 11.5 | 16.2 | 40.0 | +177.5% | 19.4 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 826007 | 24224 | 1120615 | 15516 | 20342 | 158613 | +35.7% | 14.5 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 119.2 | 39.0 | 546.8 | 12.4 | 28.9 | 41.2 | +358.5% | 14.8 | **N better** |
| N vs P+ | c2_q1_p99_us | 394.0 | 100.6 | 5095 | 882.9 | 628.4 | 2700 | +1193.2% | 7.48 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 130.0 | 37.9 | 493.7 | 22.8 | 31.2 | 72.7 | +279.9% | 11.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 138.4 | 15.1 | 1607 | 75.5 | 54.4 | 132.2 | +1060.6% | 27.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 863.7 | 341.0 | 6752 | 1759 | 1267 | 1011 | +681.7% | 4.65 | **N better** |
| N vs P+ | c4_q2_p50_us | 169.1 | 21.3 | 1454 | 97.4 | 70.5 | 97.1 | +759.9% | 18.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1750 | 651.4 | 8854 | 657.5 | 654.4 | 2115 | +405.8% | 10.9 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 720.9 | 105.2 | 1101 | 61.0 | 86.0 | 243.3 | +52.7% | 4.42 | **N better** |
| N vs P+ | write_p99_us | 2570 | 648.0 | 3349 | 179.4 | 475.4 | 4213 | +30.4% | 1.64 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 1187 | 121.9 | 774.1 | 18.3 | 87.1 | 140.3 | -34.8% | 4.74 | **N better** |
| N vs P+ | pss_mib | 825.6 | 21.0 | 491.7 | 1.02 | 14.9 | 1.69 | -40.4% | 22.4 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 146443 | 33635 | 55853 | 14507 | 25901 | 339637 | -61.9% | 3.50 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 154.3 | 15.3 | 152.5 | 5.92 | 11.6 | 8.21 | -1.2% | 0.16 | BELOW FLOOR |
| N vs P | q1_p99_us | 1788 | 800.9 | 1871 | 909.4 | 856.8 | 1239 | +4.6% | 0.10 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 177.6 | 19.9 | 236.5 | 7.78 | 15.1 | 10.8 | +33.1% | 3.90 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 826007 | 24224 | 224.1 | 35.8 | 17129 | 139706 | -100.0% | 48.2 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 119.2 | 39.0 | 85.2 | 23.3 | 32.1 | 77.1 | -28.6% | 1.06 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 394.0 | 100.6 | 1261 | 280.2 | 210.5 | 1072 | +220.0% | 4.12 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 130.0 | 37.9 | 194.8 | 27.6 | 33.1 | 125.2 | +49.9% | 1.96 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p50_us | 138.4 | 15.1 | 184.6 | 42.6 | 32.0 | 32.6 | +33.3% | 1.44 | no difference |
| N vs P | c4_q1_p99_us | 863.7 | 341.0 | 3720 | 577.6 | 474.3 | 373.0 | +330.7% | 6.02 | **N better** |
| N vs P | c4_q2_p50_us | 169.1 | 21.3 | 372.1 | 21.4 | 21.4 | 83.2 | +120.0% | 9.49 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 1750 | 651.4 | 12279 | 5604 | 3989 | 7153 | +601.5% | 2.64 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 720.9 | 105.2 | 1619100 | 19586 | 13850 | 25613 | +224493.9% | 116.9 | **N better** |
| N vs P | write_p99_us | 2570 | 648.0 | 1788008 | 46183 | 32660 | 41046 | +69485.5% | 54.7 | **N better** |
| N vs P | commits_per_s | 1187 | 121.9 | 0.61 | 0.01 | 86.2 | 63.3 | -99.9% | 13.8 | **N better** |
| N vs P | pss_mib | 825.6 | 21.0 | 484.5 | 1.53 | 14.9 | 4.59 | -41.3% | 22.9 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 146443 | 33635 | 54107 | 13188 | 25547 | 389718 | -63.1% | 3.61 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 548.0 | 9.00 | 152.5 | 5.92 | 7.61 | 99.2 | -72.2% | 51.9 | **P better** |
| P+ vs P | q1_p99_us | 4517 | 474.8 | 1871 | 909.4 | 725.4 | 931.6 | -58.6% | 3.65 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 492.9 | 11.5 | 236.5 | 7.78 | 9.83 | 40.3 | -52.0% | 26.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 277.5 | 24.9 | 240.9 | 15.3 | 20.7 | 173.1 | -13.2% | 1.77 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q5_p50_us | 1120615 | 15516 | 224.1 | 35.8 | 10971 | 75102 | -100.0% | 102.1 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 546.8 | 12.4 | 85.2 | 23.3 | 18.7 | 87.4 | -84.4% | 24.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5095 | 882.9 | 1261 | 280.2 | 655.0 | 2544 | -75.3% | 5.85 | **P better** |
| P+ vs P | c2_q2_p50_us | 493.7 | 22.8 | 194.8 | 27.6 | 25.3 | 101.9 | -60.6% | 11.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1607 | 75.5 | 184.6 | 42.6 | 61.3 | 128.3 | -88.5% | 23.2 | **P better** |
| P+ vs P | c4_q1_p99_us | 6752 | 1759 | 3720 | 577.6 | 1309 | 1054 | -44.9% | 2.32 | no difference |
| P+ vs P | c4_q2_p50_us | 1454 | 97.4 | 372.1 | 21.4 | 70.5 | 51.0 | -74.4% | 15.3 | **P better** |
| P+ vs P | c4_q2_p99_us | 8854 | 657.5 | 12279 | 5604 | 3990 | 7190 | +38.7% | 0.86 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 1101 | 61.0 | 1619100 | 19586 | 13850 | 25614 | +146973.9% | 116.8 | **P+ better** |
| P+ vs P | write_p99_us | 3349 | 179.4 | 1788008 | 46183 | 32657 | 41248 | +53283.6% | 54.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 774.1 | 18.3 | 0.61 | 0.01 | 13.0 | 125.2 | -99.9% | 59.7 | **P+ better** |
| P+ vs P | pss_mib | 491.7 | 1.02 | 484.5 | 1.53 | 1.30 | 4.89 | -1.4% | 5.47 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 55853 | 14507 | 54107 | 13188 | 13863 | 269260 | -3.1% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 152.5 | 5.92 | 149.8 | 6.77 | 6.36 | 34.4 | -1.8% | 0.42 | BELOW FLOOR |
| P vs M | q1_p99_us | 1871 | 909.4 | 1444 | 629.2 | 781.9 | 750.1 | -22.8% | 0.55 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 236.5 | 7.78 | 240.5 | 21.0 | 15.8 | 33.1 | +1.7% | 0.26 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 240.9 | 15.3 | 260.3 | 20.0 | 17.8 | 117.9 | +8.1% | 1.09 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q5_p50_us | 224.1 | 35.8 | 197.0 | 14.7 | 27.3 | 127.7 | -12.1% | 0.99 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 85.2 | 23.3 | 76.1 | 15.3 | 19.7 | 78.4 | -10.7% | 0.46 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1261 | 280.2 | 1146 | 90.3 | 208.2 | 431.1 | -9.1% | 0.55 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 194.8 | 27.6 | 209.4 | 35.1 | 31.6 | 105.9 | +7.5% | 0.46 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 184.6 | 42.6 | 184.7 | 40.7 | 41.7 | 38.3 | +0.0% | 0.00 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 3720 | 577.6 | 3361 | 660.2 | 620.3 | 1675 | -9.7% | 0.58 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 372.1 | 21.4 | 384.8 | 37.9 | 30.8 | 62.1 | +3.4% | 0.41 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 12279 | 5604 | 9944 | 5077 | 5347 | 7021 | -19.0% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 1619100 | 19586 | 1603228 | 13973 | 17013 | 47999 | -1.0% | 0.93 | BELOW FLOOR |
| P vs M | write_p99_us | 1788008 | 46183 | 1736729 | 37124 | 41899 | 42543 | -2.9% | 1.22 | no difference |
| P vs M | commits_per_s | 0.61 | 0.01 | 0.62 | 0.01 | 0.01 | 0.01 | +1.8% | 1.52 | no difference |
| P vs M | pss_mib | 484.5 | 1.53 | 484.7 | 0.68 | 1.19 | 16.5 | +0.0% | 0.10 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 54107 | 13188 | 53677 | 13958 | 13579 | 252295 | -0.8% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 154.3 | 15.3 | 322.7 | 10.4 | 13.1 | 66.4 | +109.1% | 12.9 | **N better** |
| N vs H3 | q1_p99_us | 1788 | 800.9 | 3576 | 1457 | 1176 | 7847 | +100.0% | 1.52 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 177.6 | 19.9 | 305.6 | 17.3 | 18.6 | 26.3 | +72.0% | 6.86 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 826007 | 24224 | 1130377 | 38616 | 32234 | 144165 | +36.8% | 9.44 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 119.2 | 39.0 | 311.1 | 54.6 | 47.5 | 14.8 | +160.9% | 4.04 | **N better** |
| N vs H3 | c2_q1_p99_us | 394.0 | 100.6 | 1765 | 738.0 | 526.7 | 2257 | +347.9% | 2.60 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q2_p50_us | 130.0 | 37.9 | 287.5 | 34.2 | 36.1 | 135.7 | +121.2% | 4.37 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q1_p50_us | 138.4 | 15.1 | 399.5 | 62.4 | 45.4 | 32.4 | +188.6% | 5.75 | **N better** |
| N vs H3 | c4_q1_p99_us | 863.7 | 341.0 | 4711 | 2641 | 1883 | 3496 | +445.5% | 2.04 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q2_p50_us | 169.1 | 21.3 | 422.8 | 100.1 | 72.4 | 83.6 | +150.0% | 3.50 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p99_us | 1750 | 651.4 | 5136 | 1439 | 1117 | 1716 | +193.4% | 3.03 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p50_us | 720.9 | 105.2 | 1725 | 187.0 | 151.8 | 607.1 | +139.3% | 6.62 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | write_p99_us | 2570 | 648.0 | 7650 | 3659 | 2627 | 828.5 | +197.7% | 1.93 | no difference |
| N vs H3 | commits_per_s | 1187 | 121.9 | 432.0 | 68.5 | 98.9 | 73.0 | -63.6% | 7.64 | **N better** |
| N vs H3 | pss_mib | 825.6 | 21.0 | 523.7 | 0.38 | 14.9 | 3.05 | -36.6% | 20.3 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 146443 | 33635 | 62936 | 15764 | 26266 | 352773 | -57.0% | 3.18 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 548.0 | 9.00 | 322.7 | 10.4 | 9.74 | 119.1 | -41.1% | 23.1 | **H3 better** |
| P+ vs H3 | q1_p99_us | 4517 | 474.8 | 3576 | 1457 | 1084 | 7804 | -20.8% | 0.87 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q2_p50_us | 492.9 | 11.5 | 305.6 | 17.3 | 14.7 | 46.9 | -38.0% | 12.7 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 277.5 | 24.9 | 341.0 | 29.9 | 27.5 | 136.3 | +22.9% | 2.31 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q5_p50_us | 1120615 | 15516 | 1130377 | 38616 | 29427 | 83102 | +0.9% | 0.33 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 546.8 | 12.4 | 311.1 | 54.6 | 39.6 | 43.8 | -43.1% | 5.95 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 5095 | 882.9 | 1765 | 738.0 | 813.7 | 3228 | -65.4% | 4.09 | **H3 better** |
| P+ vs H3 | c2_q2_p50_us | 493.7 | 22.8 | 287.5 | 34.2 | 29.1 | 114.6 | -41.8% | 7.10 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1607 | 75.5 | 399.5 | 62.4 | 69.3 | 128.2 | -75.1% | 17.4 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 6752 | 1759 | 4711 | 2641 | 2244 | 3633 | -30.2% | 0.91 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1454 | 97.4 | 422.8 | 100.1 | 98.8 | 51.7 | -70.9% | 10.4 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8854 | 657.5 | 5136 | 1439 | 1119 | 1863 | -42.0% | 3.32 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1101 | 61.0 | 1725 | 187.0 | 139.1 | 646.0 | +56.7% | 4.49 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | write_p99_us | 3349 | 179.4 | 7650 | 3659 | 2590 | 4163 | +128.4% | 1.66 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 774.1 | 18.3 | 432.0 | 68.5 | 50.1 | 130.4 | -44.2% | 6.82 | **P+ better** |
| P+ vs H3 | pss_mib | 491.7 | 1.02 | 523.7 | 0.38 | 0.77 | 3.47 | +6.5% | 41.5 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 55853 | 14507 | 62936 | 15764 | 15148 | 212296 | +12.7% | 0.47 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 154.3 | 15.3 | 138.8 | 11.4 | 13.5 | 16.9 | -10.0% | 1.15 | BELOW FLOOR |
| N vs H2 | q1_p99_us | 1788 | 800.9 | 1796 | 847.1 | 824.3 | 1151 | +0.4% | 0.01 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | q2_p50_us | 177.6 | 19.9 | 281.6 | 20.4 | 20.1 | 20.4 | +58.5% | 5.17 | **N better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 826007 | 24224 | 181.3 | 11.9 | 17129 | 139706 | -100.0% | 48.2 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 119.2 | 39.0 | 139.6 | 10.9 | 28.6 | 11.0 | +17.1% | 0.71 | no difference |
| N vs H2 | c2_q1_p99_us | 394.0 | 100.6 | 1841 | 237.7 | 182.5 | 1551 | +367.2% | 7.93 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 130.0 | 37.9 | 248.0 | 44.9 | 41.6 | 150.7 | +90.8% | 2.84 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q1_p50_us | 138.4 | 15.1 | 177.1 | 31.3 | 24.6 | 35.0 | +27.9% | 1.57 | no difference |
| N vs H2 | c4_q1_p99_us | 863.7 | 341.0 | 3712 | 491.2 | 422.9 | 1235 | +329.8% | 6.74 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p50_us | 169.1 | 21.3 | 396.6 | 37.4 | 30.4 | 85.1 | +134.5% | 7.47 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p99_us | 1750 | 651.4 | 10912 | 4406 | 3149 | 4451 | +523.4% | 2.91 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | write_p50_us | 720.9 | 105.2 | 2060 | 113.7 | 109.6 | 243.1 | +185.7% | 12.2 | **N better** |
| N vs H2 | write_p99_us | 2570 | 648.0 | 5402 | 283.4 | 500.1 | 1543 | +110.2% | 5.66 | **N better** |
| N vs H2 | commits_per_s | 1187 | 121.9 | 440.0 | 29.1 | 88.6 | 74.4 | -62.9% | 8.44 | **N better** |
| N vs H2 | pss_mib | 825.6 | 21.0 | 485.9 | 1.38 | 14.9 | 4.58 | -41.1% | 22.8 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 146443 | 33635 | 53852 | 13229 | 25557 | 343815 | -63.2% | 3.62 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 548.0 | 9.00 | 138.8 | 11.4 | 10.3 | 100.3 | -74.7% | 39.8 | **H2 better** |
| P+ vs H2 | q1_p99_us | 4517 | 474.8 | 1796 | 847.1 | 686.7 | 811.0 | -60.2% | 3.96 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q2_p50_us | 492.9 | 11.5 | 281.6 | 20.4 | 16.5 | 43.9 | -42.9% | 12.8 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 277.5 | 24.9 | 264.1 | 19.6 | 22.4 | 148.0 | -4.8% | 0.60 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q5_p50_us | 1120615 | 15516 | 181.3 | 11.9 | 10971 | 75102 | -100.0% | 102.1 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 546.8 | 12.4 | 139.6 | 10.9 | 11.7 | 42.6 | -74.5% | 34.9 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 5095 | 882.9 | 1841 | 237.7 | 646.6 | 2780 | -63.9% | 5.03 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q2_p50_us | 493.7 | 22.8 | 248.0 | 44.9 | 35.6 | 132.0 | -49.8% | 6.90 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p50_us | 1607 | 75.5 | 177.1 | 31.3 | 57.8 | 128.9 | -89.0% | 24.7 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 6752 | 1759 | 3712 | 491.2 | 1291 | 1580 | -45.0% | 2.35 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p50_us | 1454 | 97.4 | 396.6 | 37.4 | 73.8 | 54.0 | -72.7% | 14.3 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 8854 | 657.5 | 10912 | 4406 | 3150 | 4509 | +23.2% | 0.65 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1101 | 61.0 | 2060 | 113.7 | 91.2 | 328.5 | +87.1% | 10.5 | **P+ better** |
| P+ vs H2 | write_p99_us | 3349 | 179.4 | 5402 | 283.4 | 237.2 | 4362 | +61.3% | 8.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 774.1 | 18.3 | 440.0 | 29.1 | 24.3 | 131.2 | -43.2% | 13.7 | **P+ better** |
| P+ vs H2 | pss_mib | 491.7 | 1.02 | 485.9 | 1.38 | 1.21 | 4.87 | -1.2% | 4.76 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | 55853 | 14507 | 53852 | 13229 | 13883 | 197052 | -3.6% | 0.14 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 142.9 | 11.8 | 564.0 | 22.0 | 17.7 | 21.6 | +294.7% | 23.8 | **N better** |
| N vs P+ | q1_p99_us | 1377 | 417.8 | 5013 | 566.9 | 498.0 | 1565 | +264.0% | 7.30 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 159.1 | 15.7 | 484.5 | 12.0 | 14.0 | 40.1 | +204.5% | 23.3 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 790185 | 22603 | 1145731 | 14115 | 18843 | 101383 | +45.0% | 18.9 | **N better** |
| N vs P+ | q6_p50_us | 1430864 | 33930 | 1176486 | 18147 | 27208 | 160033 | -17.8% | 9.35 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 73.5 | 14.3 | 534.1 | 46.9 | 34.6 | 105.7 | +626.6% | 13.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 224.2 | 68.3 | 4871 | 598.6 | 426.0 | 3380 | +2072.4% | 10.9 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 95.2 | 26.6 | 523.3 | 69.2 | 52.4 | 31.6 | +449.6% | 8.17 | **N better** |
| N vs P+ | c4_q1_p50_us | 137.7 | 23.3 | 1616 | 57.7 | 44.0 | 271.1 | +1074.0% | 33.6 | **N better** |
| N vs P+ | c4_q1_p99_us | 753.1 | 248.2 | 8524 | 493.5 | 390.6 | 3825 | +1031.9% | 19.9 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 151.7 | 37.4 | 1543 | 85.3 | 65.8 | 141.4 | +916.8% | 21.1 | **N better** |
| N vs P+ | c4_q2_p99_us | 4683 | 3822 | 8554 | 1671 | 2949 | 2612 | +82.7% | 1.31 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 683.3 | 110.8 | 1090 | 42.3 | 83.9 | 397.3 | +59.6% | 4.85 | **N better** |
| N vs P+ | write_p99_us | 2446 | 725.2 | 3574 | 464.3 | 608.9 | 3377 | +46.1% | 1.85 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 1174 | 139.2 | 698.2 | 87.4 | 116.2 | 61.9 | -40.5% | 4.10 | **N better** |
| N vs P+ | pss_mib | 850.4 | 12.3 | 492.0 | 0.49 | 8.70 | 38.5 | -42.1% | 41.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 157058 | 37155 | 56094 | 13843 | 28037 | 391023 | -64.3% | 3.60 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 142.9 | 11.8 | 152.3 | 6.37 | 9.50 | 49.4 | +6.6% | 0.99 | BELOW FLOOR |
| N vs P | q1_p99_us | 1377 | 417.8 | 1258 | 495.5 | 458.3 | 590.8 | -8.7% | 0.26 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 159.1 | 15.7 | 252.8 | 13.6 | 14.7 | 23.0 | +58.9% | 6.38 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 790185 | 22603 | 203.5 | 7.41 | 15983 | 100995 | -100.0% | 49.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 1430864 | 33930 | 32181 | 1942 | 24032 | 153945 | -97.8% | 58.2 | **P better** |
| N vs P | c2_q1_p50_us | 73.5 | 14.3 | 64.5 | 5.87 | 10.9 | 40.9 | -12.2% | 0.82 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 224.2 | 68.3 | 1380 | 255.0 | 186.6 | 739.5 | +515.4% | 6.19 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 95.2 | 26.6 | 185.1 | 19.8 | 23.4 | 59.9 | +94.4% | 3.83 | **N better** |
| N vs P | c4_q1_p50_us | 137.7 | 23.3 | 157.3 | 41.3 | 33.6 | 26.2 | +14.2% | 0.58 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 753.1 | 248.2 | 3115 | 747.9 | 557.2 | 1351 | +313.7% | 4.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 151.7 | 37.4 | 309.9 | 42.0 | 39.7 | 78.7 | +104.2% | 3.98 | **N better** |
| N vs P | c4_q2_p99_us | 4683 | 3822 | 28315 | 9109 | 6985 | 21239 | +504.6% | 3.38 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 683.3 | 110.8 | 1620909 | 28892 | 20430 | 49378 | +237129.7% | 79.3 | **N better** |
| N vs P | write_p99_us | 2446 | 725.2 | 1761495 | 47148 | 33343 | 163170 | +71919.1% | 52.8 | **N better** |
| N vs P | commits_per_s | 1174 | 139.2 | 0.61 | 0.01 | 98.4 | 24.2 | -99.9% | 11.9 | **N better** |
| N vs P | pss_mib | 850.4 | 12.3 | 484.9 | 1.55 | 8.76 | 38.5 | -43.0% | 41.7 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 157058 | 37155 | 48637 | 17422 | 29018 | 397328 | -69.0% | 3.74 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 564.0 | 22.0 | 152.3 | 6.37 | 16.2 | 53.9 | -73.0% | 25.4 | **P better** |
| P+ vs P | q1_p99_us | 5013 | 566.9 | 1258 | 495.5 | 532.4 | 1514 | -74.9% | 7.05 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 484.5 | 12.0 | 252.8 | 13.6 | 12.8 | 45.1 | -47.8% | 18.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 276.3 | 21.2 | 237.6 | 20.4 | 20.8 | 46.7 | -14.0% | 1.86 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 90933 | 1714 | 6002 | 171.3 | 1218 | 5136 | -93.4% | 69.7 | **P better** |
| P+ vs P | q5_p50_us | 1145731 | 14115 | 203.5 | 7.41 | 9981 | 8860 | -100.0% | 114.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 1176486 | 18147 | 32181 | 1942 | 12905 | 43873 | -97.3% | 88.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 534.1 | 46.9 | 64.5 | 5.87 | 33.4 | 103.1 | -87.9% | 14.1 | **P better** |
| P+ vs P | c2_q1_p99_us | 4871 | 598.6 | 1380 | 255.0 | 460.0 | 3457 | -71.7% | 7.59 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 523.3 | 69.2 | 185.1 | 19.8 | 50.9 | 57.9 | -64.6% | 6.65 | **P better** |
| P+ vs P | c4_q1_p50_us | 1616 | 57.7 | 157.3 | 41.3 | 50.2 | 272.3 | -90.3% | 29.1 | **P better** |
| P+ vs P | c4_q1_p99_us | 8524 | 493.5 | 3115 | 747.9 | 633.6 | 3728 | -63.5% | 8.54 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1543 | 85.3 | 309.9 | 42.0 | 67.2 | 144.6 | -79.9% | 18.3 | **P better** |
| P+ vs P | c4_q2_p99_us | 8554 | 1671 | 28315 | 9109 | 6549 | 21374 | +231.0% | 3.02 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 1090 | 42.3 | 1620909 | 28892 | 20430 | 49377 | +148552.7% | 79.3 | **P+ better** |
| P+ vs P | write_p99_us | 3574 | 464.3 | 1761495 | 47148 | 33340 | 163202 | +49187.5% | 52.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 698.2 | 87.4 | 0.61 | 0.01 | 61.8 | 56.9 | -99.9% | 11.3 | **P+ better** |
| P+ vs P | pss_mib | 492.0 | 0.49 | 484.9 | 1.55 | 1.15 | 0.75 | -1.4% | 6.17 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 56094 | 13843 | 48637 | 17422 | 15735 | 200360 | -13.3% | 0.47 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 152.3 | 6.37 | 148.2 | 6.32 | 6.34 | 51.8 | -2.7% | 0.65 | BELOW FLOOR |
| P vs M | q1_p99_us | 1258 | 495.5 | 1117 | 470.0 | 482.9 | 324.9 | -11.2% | 0.29 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 252.8 | 13.6 | 238.9 | 12.0 | 12.8 | 22.9 | -5.5% | 1.09 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 237.6 | 20.4 | 234.2 | 29.3 | 25.3 | 61.6 | -1.4% | 0.13 | BELOW FLOOR |
| P vs M | q4_p50_us | 6002 | 171.3 | 6089 | 286.5 | 236.0 | 51.7 | +1.4% | 0.37 | no difference |
| P vs M | q5_p50_us | 203.5 | 7.41 | 199.0 | 10.9 | 9.32 | 81.5 | -2.2% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 32181 | 1942 | 32420 | 1661 | 1807 | 2745 | +0.7% | 0.13 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 64.5 | 5.87 | 63.9 | 3.79 | 4.94 | 78.3 | -0.9% | 0.12 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1380 | 255.0 | 1111 | 48.4 | 183.5 | 1016 | -19.5% | 1.47 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 185.1 | 19.8 | 166.2 | 18.6 | 19.2 | 94.1 | -10.2% | 0.98 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 157.3 | 41.3 | 172.6 | 34.4 | 38.0 | 27.9 | +9.7% | 0.40 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 3115 | 747.9 | 3430 | 170.7 | 542.4 | 741.2 | +10.1% | 0.58 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 309.9 | 42.0 | 305.5 | 55.0 | 48.9 | 86.9 | -1.4% | 0.09 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 28315 | 9109 | 21286 | 8138 | 8637 | 24314 | -24.8% | 0.81 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 1620909 | 28892 | 1608547 | 8889 | 21375 | 69506 | -0.8% | 0.58 | BELOW FLOOR |
| P vs M | write_p99_us | 1761495 | 47148 | 1752201 | 22282 | 36874 | 169901 | -0.5% | 0.25 | BELOW FLOOR |
| P vs M | commits_per_s | 0.61 | 0.01 | 0.62 | 0.01 | 0.01 | 0.02 | +1.5% | 1.20 | BELOW FLOOR |
| P vs M | pss_mib | 484.9 | 1.55 | 484.6 | 0.92 | 1.28 | 0.71 | -0.1% | 0.24 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 48637 | 17422 | 49353 | 9492 | 14029 | 210809 | +1.5% | 0.05 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 142.9 | 11.8 | 341.2 | 31.8 | 24.0 | 5.89 | +138.8% | 8.26 | **N better** |
| N vs H3 | q1_p99_us | 1377 | 417.8 | 4061 | 1523 | 1117 | 6842 | +194.9% | 2.40 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 159.1 | 15.7 | 310.7 | 7.34 | 12.3 | 7.49 | +95.3% | 12.4 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 790185 | 22603 | 1148388 | 19951 | 21318 | 102742 | +45.3% | 16.8 | **N better** |
| N vs H3 | q6_p50_us | 1430864 | 33930 | 1179350 | 23510 | 29189 | 154271 | -17.6% | 8.62 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 73.5 | 14.3 | 319.6 | 22.6 | 18.9 | 35.6 | +334.7% | 13.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c2_q1_p99_us | 224.2 | 68.3 | 2167 | 751.9 | 533.9 | 2357 | +866.5% | 3.64 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 95.2 | 26.6 | 339.2 | 49.4 | 39.7 | 37.8 | +256.2% | 6.15 | **N better** |
| N vs H3 | c4_q1_p50_us | 137.7 | 23.3 | 465.1 | 79.2 | 58.4 | 2.38 | +237.9% | 5.61 | **N better** |
| N vs H3 | c4_q1_p99_us | 753.1 | 248.2 | 4348 | 1005 | 732.3 | 3160 | +477.3% | 4.91 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 151.7 | 37.4 | 430.4 | 113.5 | 84.5 | 63.7 | +183.6% | 3.30 | **N better** |
| N vs H3 | c4_q2_p99_us | 4683 | 3822 | 8282 | 5964 | 5009 | 2746 | +76.9% | 0.72 | no difference |
| N vs H3 | write_p50_us | 683.3 | 110.8 | 1781 | 136.0 | 124.1 | 359.6 | +160.7% | 8.85 | **N better** |
| N vs H3 | write_p99_us | 2446 | 725.2 | 7991 | 3356 | 2428 | 1419 | +226.7% | 2.28 | no difference |
| N vs H3 | commits_per_s | 1174 | 139.2 | 441.9 | 82.2 | 114.3 | 116.1 | -62.4% | 6.41 | **N better** |
| N vs H3 | pss_mib | 850.4 | 12.3 | 523.3 | 0.37 | 8.70 | 38.8 | -38.5% | 37.6 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 157058 | 37155 | 62546 | 15718 | 28527 | 404538 | -60.2% | 3.31 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 564.0 | 22.0 | 341.2 | 31.8 | 27.4 | 22.4 | -39.5% | 8.14 | **H3 better** |
| P+ vs H3 | q1_p99_us | 5013 | 566.9 | 4061 | 1523 | 1149 | 6982 | -19.0% | 0.83 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 484.5 | 12.0 | 310.7 | 7.34 | 9.93 | 39.5 | -35.9% | 17.5 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 276.3 | 21.2 | 308.8 | 23.8 | 22.6 | 51.9 | +11.8% | 1.44 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 90933 | 1714 | 87213 | 3496 | 2753 | 5555 | -4.1% | 1.35 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 1145731 | 14115 | 1148388 | 19951 | 17281 | 20844 | +0.2% | 0.15 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 1176486 | 18147 | 1179350 | 23510 | 21000 | 45004 | +0.2% | 0.14 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 534.1 | 46.9 | 319.6 | 22.6 | 36.8 | 101.1 | -40.2% | 5.83 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4871 | 598.6 | 2167 | 751.9 | 679.6 | 4118 | -55.5% | 3.98 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 523.3 | 69.2 | 339.2 | 49.4 | 60.1 | 34.4 | -35.2% | 3.06 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1616 | 57.7 | 465.1 | 79.2 | 69.3 | 271.1 | -71.2% | 16.6 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 8524 | 493.5 | 4348 | 1005 | 792.0 | 4696 | -49.0% | 5.27 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c4_q2_p50_us | 1543 | 85.3 | 430.4 | 113.5 | 100.4 | 137.0 | -72.1% | 11.1 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 8554 | 1671 | 8282 | 5964 | 4380 | 3647 | -3.2% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | write_p50_us | 1090 | 42.3 | 1781 | 136.0 | 100.7 | 182.5 | +63.3% | 6.86 | **P+ better** |
| P+ vs H3 | write_p99_us | 3574 | 464.3 | 7991 | 3356 | 2396 | 3552 | +123.6% | 1.84 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | commits_per_s | 698.2 | 87.4 | 441.9 | 82.2 | 84.8 | 127.0 | -36.7% | 3.02 | **P+ better** |
| P+ vs H3 | pss_mib | 492.0 | 0.49 | 523.3 | 0.37 | 0.43 | 4.25 | +6.4% | 72.4 | no difference |
| P+ vs H3 | pss_growth_bytes_per_key_read | 56094 | 13843 | 62546 | 15718 | 14811 | 214302 | +11.5% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 142.9 | 11.8 | 136.4 | 6.92 | 9.69 | 8.80 | -4.6% | 0.67 | BELOW FLOOR |
| N vs H2 | q1_p99_us | 1377 | 417.8 | 1040 | 373.4 | 396.2 | 503.8 | -24.5% | 0.85 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 159.1 | 15.7 | 279.3 | 17.2 | 16.5 | 30.5 | +75.5% | 7.28 | **N better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 790185 | 22603 | 193.0 | 7.77 | 15983 | 100995 | -100.0% | 49.4 | **H2 better** |
| N vs H2 | q6_p50_us | 1430864 | 33930 | 29830 | 2129 | 24039 | 153925 | -97.9% | 58.3 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 73.5 | 14.3 | 143.6 | 15.9 | 15.1 | 35.6 | +95.3% | 4.64 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q1_p99_us | 224.2 | 68.3 | 1994 | 491.0 | 350.5 | 836.9 | +789.2% | 5.05 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 95.2 | 26.6 | 275.8 | 45.9 | 37.5 | 31.7 | +189.6% | 4.81 | **N better** |
| N vs H2 | c4_q1_p50_us | 137.7 | 23.3 | 163.9 | 26.6 | 25.0 | 53.8 | +19.0% | 1.05 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p99_us | 753.1 | 248.2 | 4106 | 627.6 | 477.2 | 1184 | +445.2% | 7.03 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 151.7 | 37.4 | 374.6 | 64.4 | 52.6 | 131.8 | +146.9% | 4.24 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q2_p99_us | 4683 | 3822 | 22097 | 9844 | 7467 | 12514 | +371.8% | 2.33 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | write_p50_us | 683.3 | 110.8 | 1862 | 142.7 | 127.8 | 362.2 | +172.5% | 9.22 | **N better** |
| N vs H2 | write_p99_us | 2446 | 725.2 | 5287 | 314.1 | 558.8 | 723.1 | +116.2% | 5.08 | **N better** |
| N vs H2 | commits_per_s | 1174 | 139.2 | 479.9 | 26.5 | 100.2 | 31.8 | -59.1% | 6.93 | **N better** |
| N vs H2 | pss_mib | 850.4 | 12.3 | 486.9 | 0.50 | 8.70 | 38.7 | -42.7% | 41.8 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 157058 | 37155 | 54876 | 13963 | 28067 | 395261 | -65.1% | 3.64 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 564.0 | 22.0 | 136.4 | 6.92 | 16.3 | 23.3 | -75.8% | 26.2 | **H2 better** |
| P+ vs H2 | q1_p99_us | 5013 | 566.9 | 1040 | 373.4 | 480.0 | 1482 | -79.3% | 8.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 484.5 | 12.0 | 279.3 | 17.2 | 14.8 | 49.4 | -42.4% | 13.8 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 276.3 | 21.2 | 255.3 | 32.7 | 27.6 | 34.1 | -7.6% | 0.76 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 90933 | 1714 | 6588 | 137.8 | 1216 | 5143 | -92.8% | 69.4 | **H2 better** |
| P+ vs H2 | q5_p50_us | 1145731 | 14115 | 193.0 | 7.77 | 9981 | 8860 | -100.0% | 114.8 | **H2 better** |
| P+ vs H2 | q6_p50_us | 1176486 | 18147 | 29830 | 2129 | 12920 | 43802 | -97.5% | 88.8 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 534.1 | 46.9 | 143.6 | 15.9 | 35.0 | 101.1 | -73.1% | 11.2 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 4871 | 598.6 | 1994 | 491.0 | 547.4 | 3479 | -59.1% | 5.26 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | c2_q2_p50_us | 523.3 | 69.2 | 275.8 | 45.9 | 58.7 | 27.6 | -47.3% | 4.22 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1616 | 57.7 | 163.9 | 26.6 | 44.9 | 276.4 | -89.9% | 32.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 8524 | 493.5 | 4106 | 627.6 | 564.6 | 3670 | -51.8% | 7.83 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1543 | 85.3 | 374.6 | 64.4 | 75.5 | 179.1 | -75.7% | 15.5 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p99_us | 8554 | 1671 | 22097 | 9844 | 7060 | 12742 | +158.3% | 1.92 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1090 | 42.3 | 1862 | 142.7 | 105.2 | 187.7 | +70.7% | 7.33 | **P+ better** |
| P+ vs H2 | write_p99_us | 3574 | 464.3 | 5287 | 314.1 | 396.4 | 3336 | +47.9% | 4.32 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | commits_per_s | 698.2 | 87.4 | 479.9 | 26.5 | 64.6 | 60.6 | -31.3% | 3.38 | **P+ better** |
| P+ vs H2 | pss_mib | 492.0 | 0.49 | 486.9 | 0.50 | 0.49 | 3.31 | -1.0% | 10.2 | no difference |
| P+ vs H2 | pss_growth_bytes_per_key_read | 56094 | 13843 | 54876 | 13963 | 13903 | 196228 | -2.2% | 0.09 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 139.2 | 12.1 | 558.3 | 12.6 | 12.4 | 22.0 | +301.2% | 33.9 | **N better** |
| N vs P+ | q1_p99_us | 1459 | 384.2 | 4694 | 716.0 | 574.6 | 365.7 | +221.6% | 5.63 | **N better** |
| N vs P+ | q2_p50_us | 172.6 | 14.4 | 495.6 | 27.9 | 22.2 | 48.8 | +187.1% | 14.5 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 832623 | 15010 | 1124999 | 24321 | 20209 | 32350 | +35.1% | 14.5 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 74.6 | 10.9 | 517.5 | 21.5 | 17.0 | 81.2 | +594.1% | 26.0 | **N better** |
| N vs P+ | c2_q1_p99_us | 273.9 | 106.1 | 4848 | 442.6 | 321.8 | 2664 | +1670.3% | 14.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 88.6 | 19.0 | 475.3 | 30.9 | 25.7 | 62.5 | +436.7% | 15.1 | **N better** |
| N vs P+ | c4_q1_p50_us | 119.4 | 13.2 | 1559 | 95.2 | 68.0 | 61.8 | +1205.5% | 21.2 | **N better** |
| N vs P+ | c4_q1_p99_us | 1042 | 357.7 | 7472 | 1109 | 824.3 | 3101 | +616.9% | 7.80 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 139.4 | 22.1 | 1441 | 113.7 | 81.9 | 97.8 | +934.0% | 15.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1638 | 732.8 | 9483 | 1407 | 1122 | 12720 | +478.9% | 6.99 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 675.9 | 42.9 | 1052 | 30.5 | 37.2 | 1331 | +55.6% | 10.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 2533 | 424.0 | 3479 | 300.1 | 367.3 | 315.3 | +37.3% | 2.57 | no difference |
| N vs P+ | commits_per_s | 1286 | 86.5 | 771.8 | 71.7 | 79.4 | 366.3 | -40.0% | 6.47 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 797.7 | 11.5 | 493.6 | 1.70 | 8.21 | 5.53 | -38.1% | 37.0 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 141093 | 28865 | 56064 | 13760 | 22611 | 404501 | -60.3% | 3.76 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 139.2 | 12.1 | 148.5 | 3.11 | 8.86 | 20.6 | +6.7% | 1.05 | BELOW FLOOR |
| N vs P | q1_p99_us | 1459 | 384.2 | 1918 | 690.8 | 558.9 | 345.2 | +31.5% | 0.82 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 172.6 | 14.4 | 246.2 | 14.6 | 14.5 | 53.3 | +42.6% | 5.07 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 832623 | 15010 | 219.4 | 32.6 | 10614 | 21248 | -100.0% | 78.4 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 74.6 | 10.9 | 61.2 | 1.62 | 7.81 | 43.6 | -17.9% | 1.71 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 273.9 | 106.1 | 1229 | 95.5 | 100.9 | 1727 | +348.8% | 9.47 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 88.6 | 19.0 | 190.5 | 15.2 | 17.2 | 95.4 | +115.1% | 5.93 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 119.4 | 13.2 | 163.6 | 12.7 | 13.0 | 78.8 | +37.0% | 3.41 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 1042 | 357.7 | 3009 | 391.7 | 375.1 | 888.5 | +188.7% | 5.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 139.4 | 22.1 | 329.3 | 34.4 | 28.9 | 82.2 | +136.3% | 6.57 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 1638 | 732.8 | 10615 | 4749 | 3398 | 1815 | +548.0% | 2.64 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 675.9 | 42.9 | 1592825 | 19305 | 13651 | 11764 | +235569.0% | 116.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p99_us | 2533 | 424.0 | 1686473 | 43036 | 30433 | 68615 | +66484.0% | 55.3 | **N better** |
| N vs P | commits_per_s | 1286 | 86.5 | 0.63 | 0.01 | 61.2 | 355.4 | -100.0% | 21.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_mib | 797.7 | 11.5 | 485.7 | 1.40 | 8.18 | 1.24 | -39.1% | 38.1 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 141093 | 28865 | 47493 | 8823 | 21343 | 407718 | -66.3% | 4.39 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 558.3 | 12.6 | 148.5 | 3.11 | 9.16 | 16.9 | -73.4% | 44.7 | **P better** |
| P+ vs P | q1_p99_us | 4694 | 716.0 | 1918 | 690.8 | 703.5 | 500.2 | -59.1% | 3.94 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 495.6 | 27.9 | 246.2 | 14.6 | 22.3 | 57.0 | -50.3% | 11.2 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 261.3 | 14.9 | 259.9 | 32.6 | 25.4 | 80.3 | -0.5% | 0.06 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 89865 | 2645 | 5910 | 51.5 | 1870 | 3321 | -93.4% | 44.9 | **P better** |
| P+ vs P | q5_p50_us | 1124999 | 24321 | 219.4 | 32.6 | 17198 | 24393 | -100.0% | 65.4 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 517.5 | 21.5 | 61.2 | 1.62 | 15.2 | 89.9 | -88.2% | 30.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 4848 | 442.6 | 1229 | 95.5 | 320.2 | 3163 | -74.6% | 11.3 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 475.3 | 30.9 | 190.5 | 15.2 | 24.4 | 112.6 | -59.9% | 11.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1559 | 95.2 | 163.6 | 12.7 | 67.9 | 100.1 | -89.5% | 20.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 7472 | 1109 | 3009 | 391.7 | 832.0 | 3115 | -59.7% | 5.36 | **P better** |
| P+ vs P | c4_q2_p50_us | 1441 | 113.7 | 329.3 | 34.4 | 84.0 | 95.7 | -77.2% | 13.2 | **P better** |
| P+ vs P | c4_q2_p99_us | 9483 | 1407 | 10615 | 4749 | 3502 | 12687 | +11.9% | 0.32 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 1052 | 30.5 | 1592825 | 19305 | 13651 | 11691 | +151326.8% | 116.6 | **P+ better** |
| P+ vs P | write_p99_us | 3479 | 300.1 | 1686473 | 43036 | 30432 | 68614 | +48380.7% | 55.3 | **P+ better** |
| P+ vs P | commits_per_s | 771.8 | 71.7 | 0.63 | 0.01 | 50.7 | 88.4 | -99.9% | 15.2 | **P+ better** |
| P+ vs P | pss_mib | 493.6 | 1.70 | 485.7 | 1.40 | 1.56 | 5.67 | -1.6% | 5.05 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 56064 | 13760 | 47493 | 8823 | 11558 | 195643 | -15.3% | 0.74 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 148.5 | 3.11 | 146.5 | 5.48 | 4.46 | 16.3 | -1.3% | 0.44 | BELOW FLOOR |
| P vs M | q1_p99_us | 1918 | 690.8 | 1525 | 629.7 | 661.0 | 983.9 | -20.5% | 0.59 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 246.2 | 14.6 | 266.8 | 17.1 | 15.9 | 44.4 | +8.4% | 1.30 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 259.9 | 32.6 | 264.9 | 12.2 | 24.6 | 82.3 | +1.9% | 0.20 | BELOW FLOOR |
| P vs M | q4_p50_us | 5910 | 51.5 | 5962 | 68.7 | 60.7 | 132.2 | +0.9% | 0.86 | BELOW FLOOR |
| P vs M | q5_p50_us | 219.4 | 32.6 | 196.9 | 8.53 | 23.8 | 101.3 | -10.3% | 0.94 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 61.2 | 1.62 | 60.8 | 2.09 | 1.87 | 45.5 | -0.7% | 0.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1229 | 95.5 | 1214 | 138.0 | 118.7 | 1716 | -1.2% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 190.5 | 15.2 | 166.5 | 12.4 | 13.9 | 95.3 | -12.6% | 1.73 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 163.6 | 12.7 | 157.4 | 25.7 | 20.2 | 94.5 | -3.8% | 0.31 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 3009 | 391.7 | 3334 | 355.4 | 374.0 | 2113 | +10.8% | 0.87 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 329.3 | 34.4 | 325.5 | 23.7 | 29.5 | 58.7 | -1.2% | 0.13 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 10615 | 4749 | 9815 | 3919 | 4354 | 1248 | -7.5% | 0.18 | BELOW FLOOR |
| P vs M | write_p50_us | 1592825 | 19305 | 1579336 | 33995 | 27644 | 18970 | -0.8% | 0.49 | BELOW FLOOR |
| P vs M | write_p99_us | 1686473 | 43036 | 1734420 | 31038 | 37520 | 69223 | +2.8% | 1.28 | BELOW FLOOR |
| P vs M | commits_per_s | 0.63 | 0.01 | 0.63 | 0.01 | 0.01 | 0.01 | -0.3% | 0.25 | BELOW FLOOR |
| P vs M | pss_mib | 485.7 | 1.40 | 486.6 | 0.80 | 1.14 | 5.63 | +0.2% | 0.79 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 47493 | 8823 | 49968 | 14305 | 11884 | 201233 | +5.2% | 0.21 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 139.2 | 12.1 | 331.2 | 17.0 | 14.8 | 52.3 | +138.0% | 13.0 | **N better** |
| N vs H3 | q1_p99_us | 1459 | 384.2 | 2685 | 904.7 | 695.0 | 291.5 | +84.0% | 1.76 | no difference |
| N vs H3 | q2_p50_us | 172.6 | 14.4 | 311.5 | 22.6 | 18.9 | 37.7 | +80.4% | 7.33 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 832623 | 15010 | 1128083 | 33127 | 25717 | 54467 | +35.5% | 11.5 | **N better** |
| N vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | c2_q1_p50_us | 74.6 | 10.9 | 290.5 | 29.7 | 22.4 | 41.8 | +289.7% | 9.65 | **N better** |
| N vs H3 | c2_q1_p99_us | 273.9 | 106.1 | 2364 | 847.0 | 603.6 | 21199 | +763.2% | 3.46 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 88.6 | 19.0 | 263.7 | 41.7 | 32.4 | 12.9 | +197.7% | 5.40 | **N better** |
| N vs H3 | c4_q1_p50_us | 119.4 | 13.2 | 484.2 | 127.1 | 90.4 | 73.1 | +305.5% | 4.04 | **N better** |
| N vs H3 | c4_q1_p99_us | 1042 | 357.7 | 6221 | 1751 | 1264 | 7082 | +496.8% | 4.10 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p50_us | 139.4 | 22.1 | 446.4 | 114.7 | 82.6 | 134.9 | +220.3% | 3.72 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c4_q2_p99_us | 1638 | 732.8 | 5568 | 2645 | 1941 | 4402 | +239.9% | 2.02 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 675.9 | 42.9 | 1603 | 55.2 | 49.4 | 1408 | +137.2% | 18.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | write_p99_us | 2533 | 424.0 | 7620 | 3014 | 2152 | 1242 | +200.9% | 2.36 | no difference |
| N vs H3 | commits_per_s | 1286 | 86.5 | 414.4 | 44.8 | 68.9 | 355.7 | -67.8% | 12.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 797.7 | 11.5 | 544.9 | 19.5 | 16.0 | 1.21 | -31.7% | 15.8 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 141093 | 28865 | 67582 | 12323 | 22193 | 417981 | -52.1% | 3.31 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 558.3 | 12.6 | 331.2 | 17.0 | 14.9 | 50.9 | -40.7% | 15.2 | **H3 better** |
| P+ vs H3 | q1_p99_us | 4694 | 716.0 | 2685 | 904.7 | 815.8 | 464.8 | -42.8% | 2.46 | no difference |
| P+ vs H3 | q2_p50_us | 495.6 | 27.9 | 311.5 | 22.6 | 25.4 | 42.8 | -37.1% | 7.25 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 261.3 | 14.9 | 359.0 | 47.0 | 34.8 | 113.4 | +37.4% | 2.80 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | q4_p50_us | 89865 | 2645 | 88776 | 1150 | 2039 | 4223 | -1.2% | 0.53 | BELOW FLOOR |
| P+ vs H3 | q5_p50_us | 1124999 | 24321 | 1128083 | 33127 | 29059 | 55769 | +0.3% | 0.11 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | c2_q1_p50_us | 517.5 | 21.5 | 290.5 | 29.7 | 25.9 | 89.1 | -43.9% | 8.76 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4848 | 442.6 | 2364 | 847.0 | 675.8 | 21364 | -51.2% | 3.68 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | c2_q2_p50_us | 475.3 | 30.9 | 263.7 | 41.7 | 36.7 | 61.2 | -44.5% | 5.76 | **H3 better** |
| P+ vs H3 | c4_q1_p50_us | 1559 | 95.2 | 484.2 | 127.1 | 112.3 | 95.6 | -68.9% | 9.57 | **H3 better** |
| P+ vs H3 | c4_q1_p99_us | 7472 | 1109 | 6221 | 1751 | 1466 | 7686 | -16.7% | 0.85 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p50_us | 1441 | 113.7 | 446.4 | 114.7 | 114.2 | 143.5 | -69.0% | 8.71 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q2_p99_us | 9483 | 1407 | 5568 | 2645 | 2118 | 13306 | -41.3% | 1.85 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1052 | 30.5 | 1603 | 55.2 | 44.6 | 515.6 | +52.4% | 12.4 | **P+ better** |
| P+ vs H3 | write_p99_us | 3479 | 300.1 | 7620 | 3014 | 2142 | 1201 | +119.1% | 1.93 | no difference |
| P+ vs H3 | commits_per_s | 771.8 | 71.7 | 414.4 | 44.8 | 59.8 | 89.4 | -46.3% | 5.98 | **P+ better** |
| P+ vs H3 | pss_mib | 493.6 | 1.70 | 544.9 | 19.5 | 13.8 | 5.66 | +10.4% | 3.71 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 56064 | 13760 | 67582 | 12323 | 13061 | 216219 | +20.5% | 0.88 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 139.2 | 12.1 | 133.7 | 9.38 | 10.8 | 41.0 | -4.0% | 0.51 | BELOW FLOOR |
| N vs H2 | q1_p99_us | 1459 | 384.2 | 1709 | 712.9 | 572.7 | 106.4 | +17.1% | 0.44 | no difference |
| N vs H2 | q2_p50_us | 172.6 | 14.4 | 265.8 | 13.8 | 14.1 | 55.6 | +53.9% | 6.61 | **N better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 832623 | 15010 | 173.6 | 11.7 | 10614 | 21248 | -100.0% | 78.4 | **H2 better** |
| N vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | c2_q1_p50_us | 74.6 | 10.9 | 134.6 | 20.9 | 16.7 | 38.2 | +80.6% | 3.60 | **N better** |
| N vs H2 | c2_q1_p99_us | 273.9 | 106.1 | 2082 | 556.4 | 400.6 | 228.7 | +660.3% | 4.51 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c2_q2_p50_us | 88.6 | 19.0 | 259.3 | 45.9 | 35.1 | 76.6 | +192.8% | 4.86 | **N better** |
| N vs H2 | c4_q1_p50_us | 119.4 | 13.2 | 159.3 | 18.9 | 16.3 | 18.8 | +33.4% | 2.44 | no difference |
| N vs H2 | c4_q1_p99_us | 1042 | 357.7 | 3525 | 640.9 | 519.0 | 1888 | +238.2% | 4.78 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c4_q2_p50_us | 139.4 | 22.1 | 289.3 | 33.0 | 28.1 | 88.1 | +107.6% | 5.33 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p99_us | 1638 | 732.8 | 11810 | 4472 | 3204 | 8255 | +621.0% | 3.17 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | write_p50_us | 675.9 | 42.9 | 1828 | 59.9 | 52.1 | 1343 | +170.5% | 22.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | write_p99_us | 2533 | 424.0 | 5428 | 564.8 | 499.4 | 504.3 | +114.3% | 5.80 | **N better** |
| N vs H2 | commits_per_s | 1286 | 86.5 | 480.3 | 20.5 | 62.9 | 361.4 | -62.6% | 12.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_mib | 797.7 | 11.5 | 488.9 | 0.50 | 8.13 | 3.33 | -38.7% | 38.0 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 141093 | 28865 | 54681 | 13535 | 22543 | 407553 | -61.2% | 3.83 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 558.3 | 12.6 | 133.7 | 9.38 | 11.1 | 39.3 | -76.1% | 38.3 | **H2 better** |
| P+ vs H2 | q1_p99_us | 4694 | 716.0 | 1709 | 712.9 | 714.5 | 377.3 | -63.6% | 4.18 | **H2 better** |
| P+ vs H2 | q2_p50_us | 495.6 | 27.9 | 265.8 | 13.8 | 22.0 | 59.1 | -46.4% | 10.4 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 261.3 | 14.9 | 241.0 | 27.3 | 22.0 | 32.8 | -7.8% | 0.92 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 89865 | 2645 | 6631 | 148.9 | 1873 | 3337 | -92.6% | 44.4 | **H2 better** |
| P+ vs H2 | q5_p50_us | 1124999 | 24321 | 173.6 | 11.7 | 17198 | 24393 | -100.0% | 65.4 | **H2 better** |
| P+ vs H2 | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | c2_q1_p50_us | 517.5 | 21.5 | 134.6 | 20.9 | 21.2 | 87.4 | -74.0% | 18.1 | **H2 better** |
| P+ vs H2 | c2_q1_p99_us | 4848 | 442.6 | 2082 | 556.4 | 502.8 | 2660 | -57.1% | 5.50 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c2_q2_p50_us | 475.3 | 30.9 | 259.3 | 45.9 | 39.1 | 97.2 | -45.4% | 5.52 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1559 | 95.2 | 159.3 | 18.9 | 68.7 | 64.5 | -89.8% | 20.4 | **H2 better** |
| P+ vs H2 | c4_q1_p99_us | 7472 | 1109 | 3525 | 640.9 | 906.0 | 3533 | -52.8% | 4.36 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q2_p50_us | 1441 | 113.7 | 289.3 | 33.0 | 83.7 | 100.8 | -79.9% | 13.8 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 9483 | 1407 | 11810 | 4472 | 3315 | 15027 | +24.5% | 0.70 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1052 | 30.5 | 1828 | 59.9 | 47.5 | 294.4 | +73.8% | 16.3 | **P+ better** |
| P+ vs H2 | write_p99_us | 3479 | 300.1 | 5428 | 564.8 | 452.3 | 394.3 | +56.0% | 4.31 | **P+ better** |
| P+ vs H2 | commits_per_s | 771.8 | 71.7 | 480.3 | 20.5 | 52.7 | 109.9 | -37.8% | 5.53 | **P+ better** |
| P+ vs H2 | pss_mib | 493.6 | 1.70 | 488.9 | 0.50 | 1.25 | 6.46 | -0.9% | 3.73 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | 56064 | 13760 | 54681 | 13535 | 13648 | 195298 | -2.5% | 0.10 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

## `single`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 136.5 | 16.3 | 549.9 | 29.7 | 24.0 | 24.9 | +302.8% | 17.2 | **N better** |
| N vs P+ | q1_p99_us | 1695 | 364.5 | 3831 | 494.2 | 434.2 | 5767 | +126.1% | 4.92 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 150.9 | 9.61 | 468.8 | 15.2 | 12.7 | 29.5 | +210.8% | 25.0 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 797413 | 51360 | 1123069 | 22751 | 39720 | 46768 | +40.8% | 8.20 | **N better** |
| N vs P+ | q6_p50_us | 1346928 | 32335 | 1118768 | 55203 | 45238 | 167135 | -16.9% | 5.04 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 71.8 | 10.5 | 525.1 | 31.0 | 23.2 | 33.3 | +631.9% | 19.6 | **N better** |
| N vs P+ | c2_q1_p99_us | 263.8 | 44.1 | 4770 | 700.0 | 496.0 | 649.8 | +1708.0% | 9.09 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 80.2 | 9.16 | 479.9 | 11.9 | 10.6 | 38.0 | +498.8% | 37.7 | **N better** |
| N vs P+ | c4_q1_p50_us | 121.3 | 12.5 | 1535 | 60.5 | 43.7 | 148.3 | +1165.4% | 32.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 1555 | 440.6 | 7978 | 952.6 | 742.1 | 3395 | +413.0% | 8.65 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 139.4 | 16.6 | 1416 | 52.8 | 39.1 | 85.4 | +915.6% | 32.6 | **N better** |
| N vs P+ | c4_q2_p99_us | 3439 | 1600 | 7702 | 893.4 | 1296 | 3761 | +124.0% | 3.29 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 628.3 | 73.3 | 1032 | 112.6 | 95.0 | 220.8 | +64.2% | 4.25 | **N better** |
| N vs P+ | write_p99_us | 2235 | 665.0 | 3622 | 437.2 | 562.7 | 4058 | +62.1% | 2.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1272 | 75.4 | 734.8 | 92.9 | 84.6 | 811.2 | -42.3% | 6.35 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 841.9 | 1.16 | 491.2 | 1.71 | 1.46 | 38.2 | -41.7% | 239.7 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 154724 | 38134 | 56178 | 13794 | 28675 | 446586 | -63.7% | 3.44 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 136.5 | 16.3 | 142.3 | 3.84 | 11.9 | 24.9 | +4.3% | 0.49 | BELOW FLOOR |
| N vs P | q1_p99_us | 1695 | 364.5 | 1093 | 489.5 | 431.6 | 1296 | -35.5% | 1.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 150.9 | 9.61 | 243.1 | 16.0 | 13.2 | 22.9 | +61.1% | 7.00 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 797413 | 51360 | 193.1 | 13.1 | 36317 | 44437 | -100.0% | 22.0 | **P better** |
| N vs P | q6_p50_us | 1346928 | 32335 | 30983 | 1631 | 22893 | 145083 | -97.7% | 57.5 | **P better** |
| N vs P | c2_q1_p50_us | 71.8 | 10.5 | 59.8 | 0.98 | 7.48 | 9.18 | -16.7% | 1.60 | no difference |
| N vs P | c2_q1_p99_us | 263.8 | 44.1 | 1146 | 117.0 | 88.4 | 226.4 | +334.2% | 9.97 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 80.2 | 9.16 | 173.8 | 20.4 | 15.8 | 38.6 | +116.8% | 5.92 | **N better** |
| N vs P | c4_q1_p50_us | 121.3 | 12.5 | 135.5 | 16.0 | 14.4 | 66.4 | +11.7% | 0.99 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 1555 | 440.6 | 4181 | 243.9 | 356.1 | 895.8 | +168.8% | 7.37 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 139.4 | 16.6 | 326.5 | 27.8 | 22.9 | 110.6 | +134.2% | 8.17 | **N better** |
| N vs P | c4_q2_p99_us | 3439 | 1600 | 18207 | 4738 | 3536 | 2786 | +429.4% | 4.18 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 628.3 | 73.3 | 1595214 | 29911 | 21150 | 52012 | +253773.8% | 75.4 | **N better** |
| N vs P | write_p99_us | 2235 | 665.0 | 1689208 | 22829 | 16149 | 3993 | +75484.0% | 104.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1272 | 75.4 | 0.63 | 0.01 | 53.3 | 804.1 | -100.0% | 23.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_mib | 841.9 | 1.16 | 485.5 | 1.12 | 1.14 | 38.4 | -42.3% | 312.2 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 154724 | 38134 | 43248 | 15135 | 29011 | 442571 | -72.0% | 3.84 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 549.9 | 29.7 | 142.3 | 3.84 | 21.2 | 1.46 | -74.1% | 19.2 | **P better** |
| P+ vs P | q1_p99_us | 3831 | 494.2 | 1093 | 489.5 | 491.9 | 5619 | -71.5% | 5.57 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 468.8 | 15.2 | 243.1 | 16.0 | 15.6 | 19.5 | -48.2% | 14.5 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 288.8 | 32.5 | 264.9 | 41.5 | 37.3 | 98.0 | -8.3% | 0.64 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 89529 | 2585 | 5934 | 199.5 | 1834 | 473.7 | -93.4% | 45.6 | **P better** |
| P+ vs P | q5_p50_us | 1123069 | 22751 | 193.1 | 13.1 | 16087 | 14583 | -100.0% | 69.8 | **P better** |
| P+ vs P | q6_p50_us | 1118768 | 55203 | 30983 | 1631 | 39051 | 83069 | -97.2% | 27.9 | **P better** |
| P+ vs P | c2_q1_p50_us | 525.1 | 31.0 | 59.8 | 0.98 | 22.0 | 34.0 | -88.6% | 21.2 | **P better** |
| P+ vs P | c2_q1_p99_us | 4770 | 700.0 | 1146 | 117.0 | 501.9 | 609.2 | -76.0% | 7.22 | **P better** |
| P+ vs P | c2_q2_p50_us | 479.9 | 11.9 | 173.8 | 20.4 | 16.7 | 6.71 | -63.8% | 18.3 | **P better** |
| P+ vs P | c4_q1_p50_us | 1535 | 60.5 | 135.5 | 16.0 | 44.3 | 161.8 | -91.2% | 31.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 7978 | 952.6 | 4181 | 243.9 | 695.3 | 3440 | -47.6% | 5.46 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1416 | 52.8 | 326.5 | 27.8 | 42.2 | 115.3 | -76.9% | 25.8 | **P better** |
| P+ vs P | c4_q2_p99_us | 7702 | 893.4 | 18207 | 4738 | 3409 | 4054 | +136.4% | 3.08 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 1032 | 112.6 | 1595214 | 29911 | 21150 | 52011 | +154499.7% | 75.4 | **P+ better** |
| P+ vs P | write_p99_us | 3622 | 437.2 | 1689208 | 22829 | 16146 | 1241 | +46538.0% | 104.4 | **P+ better** |
| P+ vs P | commits_per_s | 734.8 | 92.9 | 0.63 | 0.01 | 65.7 | 106.7 | -99.9% | 11.2 | **P+ better** |
| P+ vs P | pss_mib | 491.2 | 1.71 | 485.5 | 1.12 | 1.44 | 6.16 | -1.2% | 3.91 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | 56178 | 13794 | 43248 | 15135 | 14480 | 198072 | -23.0% | 0.89 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 142.3 | 3.84 | 150.4 | 3.14 | 3.51 | 2.05 | +5.7% | 2.30 | no difference |
| P vs M | q1_p99_us | 1093 | 489.5 | 1357 | 511.8 | 500.8 | 212.0 | +24.1% | 0.53 | no difference |
| P vs M | q2_p50_us | 243.1 | 16.0 | 254.1 | 11.2 | 13.8 | 33.5 | +4.5% | 0.80 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 264.9 | 41.5 | 296.0 | 37.7 | 39.6 | 70.7 | +11.8% | 0.79 | BELOW FLOOR |
| P vs M | q4_p50_us | 5934 | 199.5 | 5963 | 106.5 | 159.9 | 370.9 | +0.5% | 0.18 | BELOW FLOOR |
| P vs M | q5_p50_us | 193.1 | 13.1 | 200.4 | 6.99 | 10.5 | 8.45 | +3.8% | 0.70 | BELOW FLOOR |
| P vs M | q6_p50_us | 30983 | 1631 | 30965 | 1599 | 1615 | 3756 | -0.1% | 0.01 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 59.8 | 0.98 | 62.2 | 2.88 | 2.16 | 9.20 | +4.1% | 1.12 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1146 | 117.0 | 1083 | 60.7 | 93.2 | 390.3 | -5.5% | 0.67 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 173.8 | 20.4 | 172.4 | 18.6 | 19.5 | 42.9 | -0.8% | 0.07 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 135.5 | 16.0 | 136.4 | 19.3 | 17.7 | 70.3 | +0.7% | 0.05 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 4181 | 243.9 | 3927 | 545.4 | 422.5 | 2865 | -6.1% | 0.60 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 326.5 | 27.8 | 314.0 | 39.5 | 34.1 | 95.6 | -3.8% | 0.37 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 18207 | 4738 | 15313 | 5402 | 5081 | 11492 | -15.9% | 0.57 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 1595214 | 29911 | 1602260 | 25852 | 27955 | 54129 | +0.4% | 0.25 | BELOW FLOOR |
| P vs M | write_p99_us | 1689208 | 22829 | 1742241 | 24087 | 23466 | 69676 | +3.1% | 2.26 | BELOW FLOOR |
| P vs M | commits_per_s | 0.63 | 0.01 | 0.62 | 0.01 | 0.01 | 0.02 | -0.3% | 0.15 | BELOW FLOOR |
| P vs M | pss_mib | 485.5 | 1.12 | 485.6 | 1.17 | 1.14 | 23.5 | +0.0% | 0.10 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 43248 | 15135 | 49892 | 8925 | 12424 | 284884 | +15.4% | 0.53 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| N vs H3 | q1_p50_us | 136.5 | 16.3 | 337.6 | 19.8 | 18.1 | 24.9 | +147.2% | 11.1 | **N better** |
| N vs H3 | q1_p99_us | 1695 | 364.5 | 2542 | 1206 | 891.1 | 4653 | +50.0% | 0.95 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | q2_p50_us | 150.9 | 9.61 | 318.0 | 13.8 | 11.9 | 22.7 | +110.8% | 14.0 | **N better** |
| N vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H3 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H3 | q5_p50_us | 797413 | 51360 | 1138288 | 34835 | 43882 | 65043 | +42.7% | 7.77 | **N better** |
| N vs H3 | q6_p50_us | 1346928 | 32335 | 1164340 | 49604 | 41869 | 157107 | -13.6% | 4.36 | **H3 better** |
| N vs H3 | c2_q1_p50_us | 71.8 | 10.5 | 325.2 | 39.9 | 29.2 | 82.8 | +353.2% | 8.68 | **N better** |
| N vs H3 | c2_q1_p99_us | 263.8 | 44.1 | 2024 | 640.8 | 454.2 | 4514 | +667.1% | 3.88 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | c2_q2_p50_us | 80.2 | 9.16 | 294.9 | 31.7 | 23.3 | 147.8 | +267.9% | 9.21 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p50_us | 121.3 | 12.5 | 404.0 | 94.8 | 67.6 | 244.9 | +233.0% | 4.18 | REFUSED (warm-up MAD above 15% of the median on H3) |
| N vs H3 | c4_q1_p99_us | 1555 | 440.6 | 5760 | 1081 | 825.2 | 2694 | +270.4% | 5.10 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | c4_q2_p50_us | 139.4 | 16.6 | 374.7 | 84.3 | 60.7 | 106.6 | +168.8% | 3.87 | **N better** |
| N vs H3 | c4_q2_p99_us | 3439 | 1600 | 5985 | 1429 | 1517 | 10585 | +74.0% | 1.68 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | write_p50_us | 628.3 | 73.3 | 1663 | 63.3 | 68.5 | 538.7 | +164.7% | 15.1 | **N better** |
| N vs H3 | write_p99_us | 2235 | 665.0 | 4773 | 718.6 | 692.3 | 16340 | +113.6% | 3.67 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| N vs H3 | commits_per_s | 1272 | 75.4 | 485.6 | 50.1 | 64.0 | 804.5 | -61.8% | 12.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H3 | pss_mib | 841.9 | 1.16 | 553.5 | 0.31 | 0.85 | 38.0 | -34.3% | 338.5 | **H3 better** |
| N vs H3 | pss_growth_bytes_per_key_read | 154724 | 38134 | 71076 | 16122 | 29276 | 455277 | -54.1% | 2.86 | REFUSED (warm-up MAD above 15% of the median on N and H3) |
| P+ vs H3 | q1_p50_us | 549.9 | 29.7 | 337.6 | 19.8 | 25.2 | 1.96 | -38.6% | 8.42 | **H3 better** |
| P+ vs H3 | q1_p99_us | 3831 | 494.2 | 2542 | 1206 | 921.8 | 7180 | -33.6% | 1.40 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | q2_p50_us | 468.8 | 15.2 | 318.0 | 13.8 | 14.5 | 19.3 | -32.2% | 10.4 | **H3 better** |
| P+ vs H3 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H3 | q3_p50_us | 288.8 | 32.5 | 352.5 | 35.9 | 34.2 | 72.6 | +22.1% | 1.86 | BELOW FLOOR |
| P+ vs H3 | q4_p50_us | 89529 | 2585 | 90168 | 2151 | 2378 | 445.8 | +0.7% | 0.27 | no difference |
| P+ vs H3 | q5_p50_us | 1123069 | 22751 | 1138288 | 34835 | 29420 | 49685 | +1.4% | 0.52 | BELOW FLOOR |
| P+ vs H3 | q6_p50_us | 1118768 | 55203 | 1164340 | 49604 | 52478 | 102635 | +4.1% | 0.87 | BELOW FLOOR |
| P+ vs H3 | c2_q1_p50_us | 525.1 | 31.0 | 325.2 | 39.9 | 35.8 | 89.0 | -38.1% | 5.59 | **H3 better** |
| P+ vs H3 | c2_q1_p99_us | 4770 | 700.0 | 2024 | 640.8 | 671.0 | 4549 | -57.6% | 4.09 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c2_q2_p50_us | 479.9 | 11.9 | 294.9 | 31.7 | 23.9 | 142.9 | -38.6% | 7.74 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p50_us | 1535 | 60.5 | 404.0 | 94.8 | 79.5 | 285.9 | -73.7% | 14.2 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | c4_q1_p99_us | 7978 | 952.6 | 5760 | 1081 | 1019 | 4276 | -27.8% | 2.18 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H3 | c4_q2_p50_us | 1416 | 52.8 | 374.7 | 84.3 | 70.3 | 111.6 | -73.5% | 14.8 | **H3 better** |
| P+ vs H3 | c4_q2_p99_us | 7702 | 893.4 | 5985 | 1429 | 1192 | 10987 | -22.3% | 1.44 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| P+ vs H3 | write_p50_us | 1032 | 112.6 | 1663 | 63.3 | 91.3 | 491.5 | +61.2% | 6.92 | **P+ better** |
| P+ vs H3 | write_p99_us | 3622 | 437.2 | 4773 | 718.6 | 594.8 | 15894 | +31.8% | 1.93 | REFUSED (warm-up MAD above 15% of the median on H3) |
| P+ vs H3 | commits_per_s | 734.8 | 92.9 | 485.6 | 50.1 | 74.6 | 109.9 | -33.9% | 3.34 | **P+ better** |
| P+ vs H3 | pss_mib | 491.2 | 1.71 | 553.5 | 0.31 | 1.23 | 3.49 | +12.7% | 50.7 | **P+ better** |
| P+ vs H3 | pss_growth_bytes_per_key_read | 56178 | 13794 | 71076 | 16122 | 15003 | 225034 | +26.5% | 0.99 | REFUSED (warm-up MAD above 15% of the median on P+ and H3) |
| N vs H2 | q1_p50_us | 136.5 | 16.3 | 131.8 | 8.84 | 13.1 | 27.6 | -3.5% | 0.36 | BELOW FLOOR |
| N vs H2 | q1_p99_us | 1695 | 364.5 | 1328 | 597.4 | 494.8 | 1296 | -21.6% | 0.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | q2_p50_us | 150.9 | 9.61 | 271.8 | 27.9 | 20.9 | 24.5 | +80.2% | 5.79 | **N better** |
| N vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs H2 | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs H2 | q5_p50_us | 797413 | 51360 | 171.5 | 14.4 | 36317 | 44437 | -100.0% | 22.0 | **H2 better** |
| N vs H2 | q6_p50_us | 1346928 | 32335 | 28649 | 980.0 | 22875 | 145091 | -97.9% | 57.6 | **H2 better** |
| N vs H2 | c2_q1_p50_us | 71.8 | 10.5 | 133.2 | 20.0 | 16.0 | 96.4 | +85.6% | 3.84 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c2_q1_p99_us | 263.8 | 44.1 | 2306 | 907.4 | 642.4 | 2015 | +774.0% | 3.18 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | c2_q2_p50_us | 80.2 | 9.16 | 235.5 | 26.8 | 20.0 | 104.0 | +193.8% | 7.76 | **N better** |
| N vs H2 | c4_q1_p50_us | 121.3 | 12.5 | 144.6 | 16.6 | 14.7 | 68.9 | +19.2% | 1.58 | REFUSED (warm-up MAD above 15% of the median on H2) |
| N vs H2 | c4_q1_p99_us | 1555 | 440.6 | 3906 | 470.8 | 455.9 | 531.7 | +151.1% | 5.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | c4_q2_p50_us | 139.4 | 16.6 | 314.4 | 49.0 | 36.6 | 65.7 | +125.5% | 4.78 | **N better** |
| N vs H2 | c4_q2_p99_us | 3439 | 1600 | 15753 | 4589 | 3436 | 5980 | +358.1% | 3.58 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| N vs H2 | write_p50_us | 628.3 | 73.3 | 1842 | 157.9 | 123.1 | 223.7 | +193.2% | 9.86 | **N better** |
| N vs H2 | write_p99_us | 2235 | 665.0 | 5684 | 850.1 | 763.1 | 4078 | +154.3% | 4.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | commits_per_s | 1272 | 75.4 | 481.6 | 49.8 | 63.9 | 804.4 | -62.2% | 12.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs H2 | pss_mib | 841.9 | 1.16 | 487.9 | 0.75 | 0.98 | 38.1 | -42.0% | 361.7 | **H2 better** |
| N vs H2 | pss_growth_bytes_per_key_read | 154724 | 38134 | 54961 | 13164 | 28526 | 445158 | -64.5% | 3.50 | REFUSED (warm-up MAD above 15% of the median on N and H2) |
| P+ vs H2 | q1_p50_us | 549.9 | 29.7 | 131.8 | 8.84 | 21.9 | 11.9 | -76.0% | 19.1 | **H2 better** |
| P+ vs H2 | q1_p99_us | 3831 | 494.2 | 1328 | 597.4 | 548.2 | 5619 | -65.3% | 4.57 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | q2_p50_us | 468.8 | 15.2 | 271.8 | 27.9 | 22.5 | 21.4 | -42.0% | 8.77 | **H2 better** |
| P+ vs H2 | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs H2 | q3_p50_us | 288.8 | 32.5 | 270.4 | 19.8 | 26.9 | 71.1 | -6.4% | 0.68 | BELOW FLOOR |
| P+ vs H2 | q4_p50_us | 89529 | 2585 | 6699 | 123.1 | 1830 | 480.2 | -92.5% | 45.3 | **H2 better** |
| P+ vs H2 | q5_p50_us | 1123069 | 22751 | 171.5 | 14.4 | 16087 | 14583 | -100.0% | 69.8 | **H2 better** |
| P+ vs H2 | q6_p50_us | 1118768 | 55203 | 28649 | 980.0 | 39040 | 83084 | -97.4% | 27.9 | **H2 better** |
| P+ vs H2 | c2_q1_p50_us | 525.1 | 31.0 | 133.2 | 20.0 | 26.1 | 101.8 | -74.6% | 15.0 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q1_p99_us | 4770 | 700.0 | 2306 | 907.4 | 810.4 | 2092 | -51.7% | 3.04 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c2_q2_p50_us | 479.9 | 11.9 | 235.5 | 26.8 | 20.7 | 96.8 | -50.9% | 11.8 | **H2 better** |
| P+ vs H2 | c4_q1_p50_us | 1535 | 60.5 | 144.6 | 16.6 | 44.4 | 162.9 | -90.6% | 31.3 | REFUSED (warm-up MAD above 15% of the median on H2) |
| P+ vs H2 | c4_q1_p99_us | 7978 | 952.6 | 3906 | 470.8 | 751.3 | 3364 | -51.0% | 5.42 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs H2 | c4_q2_p50_us | 1416 | 52.8 | 314.4 | 49.0 | 50.9 | 73.5 | -77.8% | 21.6 | **H2 better** |
| P+ vs H2 | c4_q2_p99_us | 7702 | 893.4 | 15753 | 4589 | 3306 | 6665 | +104.5% | 2.44 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |
| P+ vs H2 | write_p50_us | 1032 | 112.6 | 1842 | 157.9 | 137.1 | 37.4 | +78.5% | 5.91 | **P+ better** |
| P+ vs H2 | write_p99_us | 3622 | 437.2 | 5684 | 850.1 | 675.9 | 1493 | +56.9% | 3.05 | **P+ better** |
| P+ vs H2 | commits_per_s | 734.8 | 92.9 | 481.6 | 49.8 | 74.6 | 108.7 | -34.5% | 3.40 | **P+ better** |
| P+ vs H2 | pss_mib | 491.2 | 1.71 | 487.9 | 0.75 | 1.32 | 4.24 | -0.7% | 2.45 | BELOW FLOOR |
| P+ vs H2 | pss_growth_bytes_per_key_read | 56178 | 13794 | 54961 | 13164 | 13483 | 203787 | -2.2% | 0.09 | REFUSED (warm-up MAD above 15% of the median on P+ and H2) |

