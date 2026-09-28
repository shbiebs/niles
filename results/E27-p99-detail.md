# E27 — every cell

*Generated with `results/E27-comparator.md`, from the same point files. One row per (pair, metric) per point: medians and MADs over the ten measured runs, pooled MAD, floor (3 × pooled MAD of the warm-up runs), relative change (B − A)/A, MADs apart, verdict and, when refused, why.*

## `multi`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 160.1 | 3.70 | 560.7 | 19.4 | 14.0 | 22.0 | +250.3% | 28.7 | **N better** |
| N vs P+ | q1_p99_us | 452.1 | 57.3 | 1248 | 176.4 | 131.1 | 185.3 | +176.0% | 6.07 | **N better** |
| N vs P+ | q2_p50_us | 4508 | 136.1 | 424.3 | 12.8 | 96.7 | 202.5 | -90.6% | 42.2 | **P+ better** |
| N vs P+ | q2_p99_us | 11766 | 402.3 | 1094 | 161.4 | 306.5 | 103.0 | -90.7% | 34.8 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 123.9 | 5.26 | 1578 | 62.6 | 44.4 | 24.2 | +1173.1% | 32.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 6120 | 344.3 | 5986 | 1336 | 975.8 | 8838 | -2.2% | 0.14 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 8628 | 357.6 | 1352 | 39.2 | 254.4 | 976.4 | -84.3% | 28.6 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 25366 | 2883 | 5137 | 480.2 | 2067 | 683.5 | -79.7% | 9.79 | **P+ better** |
| N vs P+ | pss_mib | 10.8 | 0.00 | 54.9 | 0.27 | 0.19 | 4.51 | +406.2% | 231.9 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4128 | 6.28 | -3596 | 215.9 | 152.7 | 2858 | -187.1% | 50.6 | REFUSED (warm-up MAD above 15% of the median on P+) |

## `multi`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 166.0 | 2.68 | 582.6 | 23.4 | 16.7 | 77.2 | +251.0% | 25.0 | **N better** |
| N vs P+ | q1_p99_us | 429.7 | 22.8 | 1254 | 188.1 | 134.0 | 156.2 | +192.0% | 6.15 | **N better** |
| N vs P+ | q2_p50_us | 4690 | 126.3 | 439.6 | 23.3 | 90.8 | 309.7 | -90.6% | 46.8 | **P+ better** |
| N vs P+ | q2_p99_us | 12141 | 539.7 | 1003 | 102.2 | 388.4 | 2339 | -91.7% | 28.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 129.8 | 6.71 | 1641 | 90.8 | 64.4 | 182.5 | +1164.8% | 23.5 | **N better** |
| N vs P+ | c4_q1_p99_us | 5829 | 105.5 | 5971 | 753.7 | 538.1 | 588.6 | +2.4% | 0.26 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 8899 | 410.6 | 1400 | 72.2 | 294.8 | 300.2 | -84.3% | 25.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 26977 | 1185 | 4721 | 529.5 | 917.5 | 1984 | -82.5% | 24.3 | **P+ better** |
| N vs P+ | pss_mib | 10.8 | 0.00 | 54.6 | 0.05 | 0.04 | 0.30 | +403.8% | 1162 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4202 | 0.00 | -3719 | 39.9 | 28.2 | 348.5 | -188.5% | 280.7 | **P+ better** |

## `multi`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 166.9 | 2.35 | 598.3 | 33.5 | 23.7 | 12.3 | +258.4% | 18.2 | **N better** |
| N vs P+ | q1_p99_us | 431.8 | 37.6 | 1436 | 184.1 | 132.9 | 233.1 | +232.5% | 7.55 | **N better** |
| N vs P+ | q2_p50_us | 4801 | 58.9 | 456.0 | 22.6 | 44.6 | 398.4 | -90.5% | 97.5 | **P+ better** |
| N vs P+ | q2_p99_us | 13301 | 630.4 | 1195 | 168.4 | 461.4 | 1957 | -91.0% | 26.2 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 130.7 | 9.82 | 1694 | 55.2 | 39.7 | 19.6 | +1196.5% | 39.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 5978 | 439.4 | 5450 | 703.2 | 586.4 | 572.0 | -8.8% | 0.90 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 9166 | 188.1 | 1464 | 52.0 | 138.0 | 88.7 | -84.0% | 55.8 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 26330 | 1014 | 4982 | 430.5 | 778.7 | 2409 | -81.1% | 27.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | pss_mib | 10.8 | 0.00 | 54.6 | 0.05 | 0.03 | 1.02 | +403.9% | 1320 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4208 | 1.60 | -3753 | 37.3 | 26.4 | 66.5 | -189.2% | 301.9 | **P+ better** |

## `multi`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 171.2 | 6.20 | 549.0 | 20.9 | 15.4 | 38.4 | +220.7% | 24.5 | **N better** |
| N vs P+ | q1_p99_us | 462.6 | 36.7 | 1157 | 122.3 | 90.3 | 297.2 | +150.1% | 7.69 | **N better** |
| N vs P+ | q2_p50_us | 4818 | 175.4 | 419.7 | 22.5 | 125.1 | 128.5 | -91.3% | 35.2 | **P+ better** |
| N vs P+ | q2_p99_us | 13319 | 873.0 | 980.0 | 50.8 | 618.4 | 2426 | -92.6% | 20.0 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 127.7 | 14.1 | 1693 | 77.0 | 55.4 | 78.5 | +1226.5% | 28.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 5771 | 219.1 | 6351 | 1442 | 1031 | 847.5 | +10.1% | 0.56 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 8618 | 446.7 | 1442 | 76.7 | 320.5 | 908.1 | -83.3% | 22.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 26826 | 2114 | 4873 | 644.2 | 1563 | 4721 | -81.8% | 14.0 | **P+ better** |
| N vs P+ | pss_mib | 10.9 | 0.00 | 55.0 | 0.66 | 0.47 | 1.99 | +404.8% | 94.7 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4197 | 0.00 | -3215 | 522.1 | 369.2 | 1429 | -176.6% | 20.1 | REFUSED (warm-up MAD above 15% of the median on P+) |

## `multi`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 174.1 | 9.59 | 568.1 | 29.9 | 22.2 | 18.3 | +226.3% | 17.7 | **N better** |
| N vs P+ | q1_p99_us | 500.4 | 78.2 | 1286 | 181.5 | 139.7 | 295.8 | +157.0% | 5.62 | **N better** |
| N vs P+ | q2_p50_us | 4927 | 106.3 | 435.5 | 21.3 | 76.7 | 125.8 | -91.2% | 58.6 | **P+ better** |
| N vs P+ | q2_p99_us | 13335 | 954.2 | 928.3 | 120.0 | 680.0 | 2530 | -93.0% | 18.2 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 138.9 | 5.80 | 1653 | 69.0 | 49.0 | 19.4 | +1090.6% | 30.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 6567 | 384.5 | 6442 | 1965 | 1416 | 1334 | -1.9% | 0.09 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 9070 | 249.3 | 1419 | 55.4 | 180.6 | 66.7 | -84.4% | 42.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 27455 | 2037 | 5347 | 463.6 | 1477 | 1790 | -80.5% | 15.0 | **P+ better** |
| N vs P+ | pss_mib | 10.9 | 0.05 | 55.2 | 0.61 | 0.43 | 0.07 | +408.8% | 102.6 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4258 | 37.6 | -3178 | 477.4 | 338.6 | 1669 | -174.6% | 22.0 | REFUSED (warm-up MAD above 15% of the median on N) |

## `multi`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 258.1 | 4.71 | 574.1 | 19.2 | 14.0 | 6.93 | +122.4% | 22.6 | **N better** |
| N vs P+ | q1_p99_us | 669.0 | 41.0 | 1390 | 156.1 | 114.1 | 281.3 | +107.7% | 6.32 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 61834 | 2005 | 440.2 | 15.2 | 1418 | 3806 | -99.3% | 43.3 | **P+ better** |
| N vs P+ | q2_p99_us | 139490 | 2614 | 1078 | 83.3 | 1849 | 3918 | -99.2% | 74.8 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 188.8 | 3.05 | 1678 | 39.2 | 27.8 | 38.2 | +788.6% | 53.6 | **N better** |
| N vs P+ | c4_q1_p99_us | 5831 | 34.7 | 5675 | 576.1 | 408.1 | 666.0 | -2.7% | 0.38 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 118154 | 2066 | 1465 | 46.5 | 1461 | 2703 | -98.8% | 79.8 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 264340 | 8457 | 5048 | 533.1 | 5992 | 19080 | -98.1% | 43.3 | **P+ better** |
| N vs P+ | pss_mib | 77.5 | 0.05 | 101.1 | 0.79 | 0.56 | 1.27 | +30.4% | 42.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4972 | 410.5 | 4064 | 232.3 | 333.5 | 5198 | -18.3% | 2.72 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `multi`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 251.7 | 4.77 | 566.1 | 17.4 | 12.7 | 23.5 | +124.9% | 24.7 | **N better** |
| N vs P+ | q1_p99_us | 748.4 | 75.5 | 1373 | 59.5 | 68.0 | 230.8 | +83.4% | 9.19 | **N better** |
| N vs P+ | q2_p50_us | 48167 | 1003 | 421.5 | 6.38 | 709.0 | 2107 | -99.1% | 67.3 | **P+ better** |
| N vs P+ | q2_p99_us | 113644 | 6225 | 1076 | 172.9 | 4403 | 1099 | -99.1% | 25.6 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 153.4 | 9.21 | 1662 | 93.6 | 66.5 | 49.0 | +983.5% | 22.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 5801 | 155.8 | 5171 | 759.7 | 548.3 | 2255 | -10.9% | 1.15 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 95012 | 3294 | 1433 | 59.9 | 2330 | 3585 | -98.5% | 40.2 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 222997 | 4294 | 5222 | 685.4 | 3075 | 17306 | -97.7% | 70.8 | **P+ better** |
| N vs P+ | pss_mib | 77.5 | 0.08 | 103.0 | 1.17 | 0.83 | 0.11 | +32.8% | 30.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 4960 | 408.5 | 4208 | 234.4 | 333.0 | 5430 | -15.2% | 2.26 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `multi`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 253.3 | 12.3 | 616.0 | 40.3 | 29.8 | 19.6 | +143.2% | 12.2 | **N better** |
| N vs P+ | q1_p99_us | 694.5 | 58.6 | 1653 | 245.1 | 178.2 | 343.9 | +138.0% | 5.38 | **N better** |
| N vs P+ | q2_p50_us | 47874 | 1181 | 457.4 | 36.1 | 835.8 | 1035 | -99.0% | 56.7 | **P+ better** |
| N vs P+ | q2_p99_us | 112585 | 1752 | 1336 | 274.3 | 1254 | 4620 | -98.8% | 88.7 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 154.9 | 5.85 | 1723 | 73.5 | 52.1 | 139.6 | +1012.5% | 30.1 | **N better** |
| N vs P+ | c4_q1_p99_us | 5931 | 87.0 | 5771 | 776.1 | 552.2 | 997.1 | -2.7% | 0.29 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 95481 | 1964 | 1500 | 67.9 | 1390 | 5003 | -98.4% | 67.6 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 212413 | 4978 | 5242 | 881.2 | 3575 | 6634 | -97.5% | 58.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | pss_mib | 77.6 | 0.05 | 100.9 | 2.06 | 1.46 | 0.20 | +30.1% | 16.0 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 5014 | 426.8 | 4037 | 168.5 | 324.5 | 5283 | -19.5% | 3.01 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `multi`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 243.7 | 8.31 | 601.0 | 46.0 | 33.0 | 82.2 | +146.6% | 10.8 | **N better** |
| N vs P+ | q1_p99_us | 661.2 | 50.9 | 1547 | 158.6 | 117.8 | 768.5 | +134.0% | 7.52 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 46663 | 2164 | 452.8 | 35.1 | 1531 | 3148 | -99.0% | 30.2 | **P+ better** |
| N vs P+ | q2_p99_us | 115905 | 5647 | 1297 | 124.0 | 3994 | 8819 | -98.9% | 28.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 156.0 | 10.3 | 1661 | 95.4 | 67.9 | 235.5 | +964.8% | 22.2 | **N better** |
| N vs P+ | c4_q1_p99_us | 5938 | 151.5 | 5668 | 508.5 | 375.2 | 1786 | -4.5% | 0.72 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 99438 | 3679 | 1462 | 54.7 | 2602 | 9734 | -98.5% | 37.7 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 230832 | 11327 | 5720 | 591.6 | 8021 | 17906 | -97.5% | 28.1 | **P+ better** |
| N vs P+ | pss_mib | 77.5 | 0.14 | 102.9 | 0.28 | 0.22 | 0.41 | +32.8% | 114.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 5003 | 408.4 | 4247 | 271.7 | 346.8 | 5283 | -15.1% | 2.18 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `multi`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 257.6 | 6.62 | 615.5 | 38.5 | 27.6 | 101.7 | +138.9% | 13.0 | **N better** |
| N vs P+ | q1_p99_us | 827.3 | 66.8 | 1822 | 136.1 | 107.2 | 314.9 | +120.3% | 9.28 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 50075 | 2724 | 458.0 | 45.4 | 1927 | 6502 | -99.1% | 25.8 | **P+ better** |
| N vs P+ | q2_p99_us | 122031 | 958.6 | 1475 | 219.7 | 695.4 | 946.2 | -98.8% | 173.4 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 155.5 | 6.35 | 1746 | 65.8 | 46.8 | 239.3 | +1023.0% | 34.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 5871 | 58.0 | 6051 | 275.3 | 199.0 | 965.7 | +3.1% | 0.90 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 100674 | 3030 | 1473 | 26.7 | 2143 | 7200 | -98.5% | 46.3 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 228331 | 10954 | 5286 | 520.8 | 7755 | 7406 | -97.7% | 28.8 | **P+ better** |
| N vs P+ | pss_mib | 77.6 | 0.09 | 102.0 | 1.14 | 0.81 | 0.09 | +31.4% | 30.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 5003 | 443.6 | 4197 | 167.7 | 335.3 | 5403 | -16.1% | 2.40 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `multi`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 89.2 | 10.2 | 830.1 | 48.7 | 35.2 | 17.8 | +830.8% | 21.1 | **N better** |
| N vs P+ | q1_p99_us | 511.3 | 102.0 | 2256 | 363.5 | 266.9 | 1127 | +341.3% | 6.54 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 119.5 | 22.9 | 2187 | 74.7 | 55.2 | 217.1 | +1729.6% | 37.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 1021 | 101.2 | 7651 | 497.4 | 358.9 | 181.3 | +649.1% | 18.5 | **N better** |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | pss_mib | 752.8 | 0.13 | 351.3 | 24.9 | 17.6 | 3.62 | -53.3% | 22.8 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 37063 | 8546 | 2634 | 1057 | 6089 | 94639 | -92.9% | 5.65 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `multi`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 90.9 | 8.06 | 811.3 | 99.8 | 70.8 | 19.1 | +792.3% | 10.2 | **N better** |
| N vs P+ | q1_p99_us | 558.3 | 30.1 | 2249 | 392.3 | 278.2 | 508.2 | +302.8% | 6.08 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 133.8 | 30.1 | 2363 | 155.4 | 112.0 | 147.9 | +1665.5% | 19.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1005 | 140.6 | 7787 | 578.3 | 420.8 | 325.5 | +675.0% | 16.1 | **N better** |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | pss_mib | 753.1 | 0.21 | 360.7 | 26.3 | 18.6 | 3.42 | -52.1% | 21.1 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 37103 | 8458 | 2545 | 1077 | 6029 | 94708 | -93.1% | 5.73 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `multi`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 88.2 | 3.61 | 622.3 | 43.9 | 31.1 | 37.6 | +605.8% | 17.2 | **N better** |
| N vs P+ | q1_p99_us | 546.6 | 79.3 | 1586 | 110.4 | 96.1 | 94.4 | +190.1% | 10.8 | **N better** |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 157.3 | 13.1 | 2027 | 168.6 | 119.6 | 158.9 | +1188.2% | 15.6 | **N better** |
| N vs P+ | c4_q1_p99_us | 926.1 | 80.1 | 7153 | 444.9 | 319.7 | 541.2 | +672.3% | 19.5 | **N better** |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | pss_mib | 753.0 | 0.02 | 355.0 | 17.9 | 12.6 | 2.54 | -52.9% | 31.5 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 37113 | 8531 | 1775 | 1021 | 6075 | 67621 | -95.2% | 5.82 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `multi`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 92.9 | 4.83 | 824.3 | 29.7 | 21.3 | 156.0 | +787.2% | 34.3 | **N better** |
| N vs P+ | q1_p99_us | 488.0 | 83.5 | 2125 | 349.0 | 253.7 | 1294 | +335.5% | 6.45 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 171.4 | 7.59 | 2326 | 74.1 | 52.7 | 251.1 | +1256.7% | 40.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 942.7 | 62.1 | 8066 | 812.4 | 576.1 | 607.2 | +755.6% | 12.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | pss_mib | 752.8 | 0.16 | 356.8 | 25.6 | 18.1 | 6.34 | -52.6% | 21.9 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 37216 | 8692 | 3069 | 1007 | 6187 | 94704 | -91.8% | 5.52 | REFUSED (warm-up MAD above 15% of the median on N) |

## `multi`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 98.4 | 7.64 | 826.5 | 39.6 | 28.5 | 29.8 | +740.3% | 25.5 | **N better** |
| N vs P+ | q1_p99_us | 573.8 | 80.1 | 1786 | 217.4 | 163.9 | 185.0 | +211.2% | 7.40 | **N better** |
| N vs P+ | q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 89.6 | 12.3 | 2319 | 95.1 | 67.8 | 162.5 | +2488.3% | 32.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 939.7 | 204.9 | 7960 | 594.5 | 444.6 | 489.9 | +747.0% | 15.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q2_p99_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | pss_mib | 752.7 | 0.48 | 361.7 | 24.7 | 17.5 | 2.38 | -51.9% | 22.4 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 37054 | 8518 | 2719 | 1008 | 6065 | 95807 | -92.7% | 5.66 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 116.3 | 2.89 | 589.3 | 27.3 | 19.4 | 59.4 | +406.6% | 24.4 | **N better** |
| N vs P+ | q1_p99_us | 249.2 | 20.1 | 1505 | 129.0 | 92.3 | 123.5 | +504.0% | 13.6 | **N better** |
| N vs P+ | q2_p50_us | 123.2 | 5.18 | 448.2 | 11.7 | 9.02 | 49.2 | +263.8% | 36.0 | **N better** |
| N vs P+ | q2_p99_us | 439.8 | 36.5 | 1100 | 107.9 | 80.6 | 214.1 | +150.2% | 8.20 | **N better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 135.7 | 4.70 | 1755 | 61.9 | 43.9 | 207.9 | +1193.3% | 36.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 514.0 | 66.2 | 5645 | 689.4 | 489.7 | 234.2 | +998.4% | 10.5 | **N better** |
| N vs P+ | c4_q2_p50_us | 145.7 | 7.27 | 1505 | 46.9 | 33.6 | 146.7 | +932.9% | 40.5 | **N better** |
| N vs P+ | c4_q2_p99_us | 655.3 | 98.7 | 5433 | 463.3 | 334.9 | 134.4 | +729.1% | 14.3 | **N better** |
| N vs P+ | pss_mib | 15.7 | 0.04 | 54.8 | 0.37 | 0.26 | 1.50 | +248.6% | 148.6 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 10556 | 41.0 | -4813 | 388.1 | 275.9 | 2440 | -145.6% | 55.7 | REFUSED (warm-up MAD above 15% of the median on P+) |

## `single`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 116.6 | 3.67 | 583.8 | 12.0 | 8.86 | 5.42 | +400.8% | 52.7 | **N better** |
| N vs P+ | q1_p99_us | 245.0 | 29.8 | 1173 | 116.3 | 84.9 | 303.6 | +378.8% | 10.9 | **N better** |
| N vs P+ | q2_p50_us | 120.8 | 4.34 | 436.7 | 14.9 | 11.0 | 4.36 | +261.6% | 28.8 | **N better** |
| N vs P+ | q2_p99_us | 459.7 | 52.5 | 944.8 | 82.3 | 69.0 | 165.8 | +105.5% | 7.03 | **N better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 130.4 | 6.41 | 1713 | 10.2 | 8.55 | 69.8 | +1213.8% | 185.2 | **N better** |
| N vs P+ | c4_q1_p99_us | 648.1 | 149.7 | 7313 | 2453 | 1737 | 2119 | +1028.4% | 3.84 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 149.9 | 13.0 | 1479 | 38.9 | 29.0 | 85.2 | +886.4% | 45.8 | **N better** |
| N vs P+ | c4_q2_p99_us | 610.9 | 95.6 | 4889 | 468.6 | 338.1 | 276.1 | +700.4% | 12.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 15.9 | 0.03 | 54.7 | 0.20 | 0.15 | 2.13 | +244.8% | 265.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 10753 | 32.8 | -4847 | 214.5 | 153.5 | 2210 | -145.1% | 101.7 | REFUSED (warm-up MAD above 15% of the median on P+) |

## `single`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 117.2 | 1.88 | 726.2 | 27.7 | 19.6 | 33.8 | +519.4% | 31.1 | **N better** |
| N vs P+ | q1_p99_us | 279.1 | 33.2 | 1744 | 396.0 | 281.0 | 415.2 | +525.1% | 5.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 123.2 | 2.57 | 568.9 | 20.5 | 14.6 | 14.4 | +361.6% | 30.6 | **N better** |
| N vs P+ | q2_p99_us | 401.0 | 25.4 | 1440 | 188.3 | 134.3 | 93.4 | +259.1% | 7.73 | **N better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 141.7 | 4.96 | 1965 | 168.6 | 119.3 | 120.5 | +1286.7% | 15.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 600.2 | 91.6 | 8172 | 673.4 | 480.6 | 1468 | +1261.5% | 15.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 147.9 | 7.62 | 1736 | 133.0 | 94.2 | 46.2 | +1073.8% | 16.9 | **N better** |
| N vs P+ | c4_q2_p99_us | 625.5 | 76.6 | 7012 | 1175 | 832.8 | 882.2 | +1021.1% | 7.67 | **N better** |
| N vs P+ | pss_mib | 15.8 | 0.01 | 54.9 | 0.25 | 0.18 | 0.15 | +247.5% | 223.0 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 10703 | 14.3 | -4727 | 259.6 | 183.8 | 910.6 | -144.2% | 83.9 | **P+ better** |

## `single`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 118.2 | 8.04 | 572.1 | 15.8 | 12.5 | 15.2 | +384.0% | 36.2 | **N better** |
| N vs P+ | q1_p99_us | 238.2 | 24.8 | 1241 | 117.5 | 84.9 | 64.1 | +421.1% | 11.8 | **N better** |
| N vs P+ | q2_p50_us | 122.9 | 11.6 | 429.1 | 13.9 | 12.8 | 64.6 | +249.0% | 24.0 | **N better** |
| N vs P+ | q2_p99_us | 405.0 | 37.2 | 1019 | 119.4 | 88.4 | 132.5 | +151.6% | 6.94 | **N better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 142.2 | 4.97 | 1661 | 17.6 | 12.9 | 54.7 | +1068.1% | 117.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 517.7 | 53.4 | 7668 | 1905 | 1348 | 1169 | +1381.1% | 5.31 | **N better** |
| N vs P+ | c4_q2_p50_us | 149.8 | 8.36 | 1446 | 41.8 | 30.2 | 65.2 | +864.9% | 43.0 | **N better** |
| N vs P+ | c4_q2_p99_us | 601.5 | 38.6 | 6196 | 942.4 | 666.9 | 287.3 | +930.1% | 8.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 15.9 | 0.01 | 54.4 | 0.19 | 0.14 | 0.58 | +243.2% | 281.7 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 10759 | 10.2 | -4853 | 202.8 | 143.6 | 73.9 | -145.1% | 108.8 | **P+ better** |

## `single`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 116.2 | 5.11 | 567.2 | 10.9 | 8.52 | 4.36 | +388.3% | 53.0 | **N better** |
| N vs P+ | q1_p99_us | 266.2 | 38.1 | 1352 | 149.6 | 109.1 | 71.8 | +407.8% | 9.94 | **N better** |
| N vs P+ | q2_p50_us | 125.4 | 10.7 | 424.8 | 7.98 | 9.43 | 29.5 | +238.7% | 31.7 | **N better** |
| N vs P+ | q2_p99_us | 442.4 | 20.6 | 1247 | 378.4 | 268.0 | 60.4 | +182.0% | 3.00 | **N better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 137.4 | 6.07 | 1657 | 55.4 | 39.4 | 69.6 | +1106.1% | 38.6 | **N better** |
| N vs P+ | c4_q1_p99_us | 525.7 | 61.9 | 7907 | 2397 | 1695 | 138.5 | +1404.0% | 4.35 | **N better** |
| N vs P+ | c4_q2_p50_us | 143.7 | 5.89 | 1433 | 31.6 | 22.7 | 71.8 | +896.9% | 56.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 649.5 | 117.3 | 5724 | 626.3 | 450.6 | 146.1 | +781.3% | 11.3 | **N better** |
| N vs P+ | pss_mib | 15.5 | 0.03 | 54.8 | 0.23 | 0.17 | 1.02 | +252.6% | 236.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 10479 | 28.7 | -4794 | 244.7 | 174.2 | 1031 | -145.8% | 87.7 | **P+ better** |

## `single`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 211.8 | 15.5 | 599.2 | 27.7 | 22.5 | 116.4 | +182.9% | 17.3 | **N better** |
| N vs P+ | q1_p99_us | 445.3 | 39.7 | 1320 | 116.4 | 86.9 | 670.5 | +196.5% | 10.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 128.8 | 3.29 | 444.3 | 13.3 | 9.65 | 14.1 | +245.1% | 32.7 | **N better** |
| N vs P+ | q2_p99_us | 753.4 | 80.8 | 1144 | 156.2 | 124.4 | 226.2 | +51.8% | 3.14 | **N better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 354.1 | 29.9 | 1703 | 52.0 | 42.4 | 67.9 | +380.8% | 31.8 | **N better** |
| N vs P+ | c4_q1_p99_us | 1304 | 172.9 | 5401 | 437.7 | 332.8 | 493.7 | +314.2% | 12.3 | **N better** |
| N vs P+ | c4_q2_p50_us | 196.5 | 13.0 | 1458 | 45.5 | 33.4 | 121.2 | +641.8% | 37.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 1365 | 85.5 | 4867 | 471.9 | 339.2 | 547.4 | +256.6% | 10.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 83.8 | 0.19 | 54.8 | 0.25 | 0.22 | 0.13 | -34.6% | 129.7 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 6644 | 414.7 | -536.9 | 14.2 | 293.4 | 5774 | -108.1% | 24.5 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 219.1 | 10.7 | 603.0 | 9.38 | 10.1 | 32.0 | +175.2% | 38.1 | **N better** |
| N vs P+ | q1_p99_us | 507.3 | 98.6 | 1258 | 98.5 | 98.6 | 73.7 | +148.0% | 7.62 | **N better** |
| N vs P+ | q2_p50_us | 139.5 | 5.86 | 442.5 | 5.75 | 5.80 | 19.4 | +217.1% | 52.2 | **N better** |
| N vs P+ | q2_p99_us | 887.5 | 141.2 | 1021 | 135.4 | 138.3 | 87.7 | +15.0% | 0.96 | no difference |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 362.0 | 28.8 | 1788 | 41.8 | 35.9 | 163.0 | +394.1% | 39.8 | **N better** |
| N vs P+ | c4_q1_p99_us | 1461 | 247.8 | 5552 | 374.6 | 317.6 | 775.0 | +280.0% | 12.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 194.2 | 13.7 | 1535 | 41.4 | 30.8 | 88.8 | +690.3% | 43.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1414 | 62.1 | 5370 | 316.8 | 228.3 | 1083 | +279.7% | 17.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 84.6 | 0.08 | 54.3 | 0.21 | 0.16 | 0.18 | -35.8% | 189.1 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 6704 | 405.3 | -567.0 | 77.9 | 291.9 | 17337 | -108.5% | 24.9 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 211.3 | 11.5 | 582.8 | 2.95 | 8.38 | 56.2 | +175.8% | 44.3 | **N better** |
| N vs P+ | q1_p99_us | 465.0 | 38.3 | 1292 | 138.4 | 101.6 | 100.8 | +177.9% | 8.14 | **N better** |
| N vs P+ | q2_p50_us | 134.6 | 5.07 | 440.4 | 8.26 | 6.85 | 24.9 | +227.1% | 44.6 | **N better** |
| N vs P+ | q2_p99_us | 832.5 | 69.1 | 1051 | 38.2 | 55.8 | 362.5 | +26.2% | 3.91 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 361.0 | 26.2 | 1692 | 28.0 | 27.1 | 75.5 | +368.6% | 49.1 | **N better** |
| N vs P+ | c4_q1_p99_us | 1319 | 188.1 | 5517 | 484.5 | 367.5 | 157.9 | +318.4% | 11.4 | **N better** |
| N vs P+ | c4_q2_p50_us | 190.7 | 9.65 | 1469 | 37.1 | 27.1 | 28.8 | +670.5% | 47.2 | **N better** |
| N vs P+ | c4_q2_p99_us | 1216 | 172.7 | 4788 | 205.1 | 189.6 | 301.3 | +293.9% | 18.8 | **N better** |
| N vs P+ | pss_mib | 84.3 | 0.13 | 54.5 | 0.21 | 0.17 | 0.88 | -35.4% | 172.3 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 6696 | 378.4 | -553.3 | 56.9 | 270.5 | 5940 | -108.3% | 26.8 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 213.8 | 13.6 | 617.7 | 9.92 | 11.9 | 48.2 | +188.9% | 33.9 | **N better** |
| N vs P+ | q1_p99_us | 475.7 | 36.4 | 1542 | 232.0 | 166.0 | 1057 | +224.1% | 6.42 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 135.9 | 3.22 | 456.2 | 7.52 | 5.79 | 26.2 | +235.6% | 55.4 | **N better** |
| N vs P+ | q2_p99_us | 909.0 | 89.1 | 1164 | 209.1 | 160.8 | 635.1 | +28.0% | 1.58 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 349.2 | 32.2 | 1842 | 31.8 | 32.0 | 65.0 | +427.6% | 46.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 1468 | 270.8 | 5826 | 642.6 | 493.1 | 596.2 | +296.8% | 8.84 | **N better** |
| N vs P+ | c4_q2_p50_us | 186.3 | 13.4 | 1570 | 45.5 | 33.5 | 14.5 | +742.5% | 41.3 | **N better** |
| N vs P+ | c4_q2_p99_us | 1516 | 116.8 | 4982 | 701.0 | 502.5 | 1783 | +228.7% | 6.90 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | pss_mib | 84.0 | 0.38 | 54.7 | 0.40 | 0.39 | 1.06 | -34.9% | 75.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 6698 | 367.4 | -539.3 | 64.4 | 263.7 | 12713 | -108.1% | 27.4 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 227.1 | 17.8 | 647.1 | 34.5 | 27.4 | 129.5 | +184.9% | 15.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 465.2 | 59.2 | 1668 | 277.4 | 200.6 | 493.1 | +258.6% | 6.00 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 137.7 | 6.26 | 475.6 | 23.0 | 16.9 | 26.1 | +245.4% | 20.0 | **N better** |
| N vs P+ | q2_p99_us | 842.0 | 85.6 | 1185 | 88.0 | 86.8 | 228.2 | +40.7% | 3.95 | **N better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 368.7 | 30.7 | 1910 | 59.2 | 47.2 | 146.4 | +418.1% | 32.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 1482 | 233.2 | 7227 | 1152 | 831.1 | 340.9 | +387.8% | 6.91 | **N better** |
| N vs P+ | c4_q2_p50_us | 186.9 | 11.1 | 1623 | 36.1 | 26.7 | 77.0 | +768.4% | 53.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 1330 | 102.9 | 5897 | 585.2 | 420.2 | 1340 | +343.5% | 10.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 84.1 | 0.09 | 54.6 | 0.16 | 0.13 | 1.00 | -35.1% | 225.0 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 6694 | 409.3 | -530.7 | 120.9 | 301.8 | 5953 | -107.9% | 23.9 | REFUSED (warm-up MAD above 15% of the median on N) |

## `single`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1047 | 113.3 | 840.6 | 108.1 | 110.8 | 38.9 | -19.7% | 1.86 | no difference |
| N vs P+ | q1_p99_us | 1966 | 188.7 | 2114 | 457.9 | 350.2 | 654.7 | +7.6% | 0.42 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 169.2 | 13.9 | 653.5 | 97.8 | 69.9 | 11.9 | +286.2% | 6.93 | **N better** |
| N vs P+ | q2_p99_us | 1593 | 136.4 | 1672 | 385.8 | 289.3 | 167.8 | +4.9% | 0.27 | BELOW FLOOR |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 3023 | 133.2 | 2234 | 172.7 | 154.2 | 157.4 | -26.1% | 5.11 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 8242 | 609.5 | 7638 | 885.4 | 760.1 | 879.1 | -7.3% | 0.79 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 172.6 | 8.29 | 1998 | 154.2 | 109.2 | 123.2 | +1057.7% | 16.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 3159 | 153.9 | 7384 | 650.6 | 472.8 | 391.2 | +133.7% | 8.94 | **N better** |
| N vs P+ | pss_mib | 760.0 | 0.58 | 402.4 | 57.6 | 40.7 | 9.84 | -47.1% | 8.78 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 22054 | 4564 | 4285 | 1110 | 3321 | 49959 | -80.6% | 5.35 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1033 | 84.6 | 830.3 | 53.4 | 70.8 | 67.8 | -19.6% | 2.86 | no difference |
| N vs P+ | q1_p99_us | 1956 | 316.8 | 2235 | 125.9 | 241.1 | 459.0 | +14.3% | 1.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 164.3 | 6.39 | 674.9 | 47.2 | 33.7 | 25.8 | +310.7% | 15.2 | **N better** |
| N vs P+ | q2_p99_us | 1631 | 96.9 | 1738 | 193.4 | 153.0 | 438.2 | +6.5% | 0.70 | BELOW FLOOR |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 2845 | 301.0 | 2029 | 83.7 | 220.9 | 35.7 | -28.7% | 3.69 | **P+ better** |
| N vs P+ | c4_q1_p99_us | 8173 | 987.0 | 6938 | 700.5 | 855.8 | 535.3 | -15.1% | 1.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 174.8 | 4.24 | 1765 | 98.5 | 69.7 | 105.8 | +909.3% | 22.8 | **N better** |
| N vs P+ | c4_q2_p99_us | 2883 | 247.1 | 7002 | 959.0 | 700.3 | 1273 | +142.8% | 5.88 | **N better** |
| N vs P+ | pss_mib | 758.4 | 0.87 | 402.0 | 51.7 | 36.5 | 2.54 | -47.0% | 9.76 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 22101 | 4542 | 4667 | 1011 | 3290 | 48329 | -78.9% | 5.30 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1062 | 66.6 | 663.7 | 12.0 | 47.9 | 44.1 | -37.5% | 8.32 | **P+ better** |
| N vs P+ | q1_p99_us | 2004 | 228.5 | 1821 | 293.8 | 263.2 | 174.6 | -9.1% | 0.69 | no difference |
| N vs P+ | q2_p50_us | 176.5 | 8.07 | 506.3 | 29.8 | 21.8 | 22.0 | +186.9% | 15.1 | **N better** |
| N vs P+ | q2_p99_us | 1612 | 301.3 | 1299 | 174.6 | 246.2 | 341.0 | -19.4% | 1.27 | BELOW FLOOR |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 2977 | 57.8 | 1947 | 136.6 | 104.9 | 116.4 | -34.6% | 9.82 | **P+ better** |
| N vs P+ | c4_q1_p99_us | 8287 | 612.0 | 6839 | 1247 | 982.3 | 791.9 | -17.5% | 1.47 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 172.1 | 7.62 | 1643 | 128.3 | 90.9 | 65.7 | +854.9% | 16.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 3444 | 516.4 | 6438 | 489.8 | 503.3 | 972.0 | +86.9% | 5.95 | **N better** |
| N vs P+ | pss_mib | 758.9 | 1.46 | 406.6 | 48.8 | 34.5 | 4.24 | -46.4% | 10.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 22243 | 4724 | 4050 | 980.9 | 3411 | 49578 | -81.8% | 5.33 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1036 | 127.2 | 713.4 | 74.1 | 104.1 | 15.6 | -31.1% | 3.10 | **P+ better** |
| N vs P+ | q1_p99_us | 1967 | 201.4 | 1846 | 132.2 | 170.4 | 51.0 | -6.1% | 0.71 | no difference |
| N vs P+ | q2_p50_us | 159.3 | 3.27 | 547.4 | 64.1 | 45.4 | 5.71 | +243.7% | 8.55 | **N better** |
| N vs P+ | q2_p99_us | 1295 | 260.4 | 1463 | 205.8 | 234.7 | 585.7 | +13.0% | 0.72 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 2989 | 276.8 | 2032 | 204.4 | 243.3 | 78.9 | -32.0% | 3.93 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 8487 | 540.7 | 6764 | 593.9 | 568.0 | 2292 | -20.3% | 3.03 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 156.1 | 5.00 | 1818 | 93.9 | 66.5 | 91.6 | +1065.0% | 25.0 | **N better** |
| N vs P+ | c4_q2_p99_us | 3006 | 286.7 | 6003 | 618.2 | 481.9 | 1203 | +99.7% | 6.22 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 758.4 | 0.95 | 403.9 | 50.9 | 36.0 | 4.88 | -46.7% | 9.84 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 21924 | 4451 | 3917 | 1223 | 3264 | 48694 | -82.1% | 5.52 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

## `single`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 1024 | 82.6 | 766.4 | 43.9 | 66.1 | 70.3 | -25.1% | 3.89 | **P+ better** |
| N vs P+ | q1_p99_us | 1710 | 132.4 | 1765 | 247.0 | 198.2 | 193.1 | +3.2% | 0.28 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 168.7 | 5.84 | 597.0 | 44.4 | 31.7 | 37.9 | +253.8% | 13.5 | **N better** |
| N vs P+ | q2_p99_us | 1294 | 135.6 | 1511 | 260.7 | 207.8 | 1128 | +16.8% | 1.04 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | c4_q1_p50_us | 3031 | 182.4 | 2064 | 155.0 | 169.3 | 83.3 | -31.9% | 5.71 | **P+ better** |
| N vs P+ | c4_q1_p99_us | 8478 | 503.8 | 6996 | 715.4 | 618.8 | 1063 | -17.5% | 2.40 | no difference |
| N vs P+ | c4_q2_p50_us | 161.1 | 10.0 | 1830 | 98.9 | 70.3 | 113.3 | +1035.6% | 23.7 | **N better** |
| N vs P+ | c4_q2_p99_us | 3026 | 271.1 | 6537 | 290.8 | 281.1 | 1055 | +116.0% | 12.5 | **N better** |
| N vs P+ | pss_mib | 759.8 | 1.21 | 415.4 | 52.2 | 36.9 | 5.59 | -45.3% | 9.34 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 22165 | 4657 | 3908 | 1144 | 3391 | 49389 | -82.4% | 5.38 | REFUSED (warm-up MAD above 15% of the median on N and P+) |

