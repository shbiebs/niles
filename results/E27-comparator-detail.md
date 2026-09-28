# E27 — every cell

*Generated with `results/E27-comparator.md`, from the same point files. One row per (pair, metric) per point: medians and MADs over the ten measured runs, pooled MAD, floor (3 × pooled MAD of the warm-up runs), relative change (B − A)/A, MADs apart, verdict and, when refused, why.*

## `multi`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 144.8 | 2.61 | 565.0 | 24.4 | 17.3 | 12.3 | +290.3% | 24.2 | **N better** |
| N vs P+ | q1_p99_us | 297.6 | 33.7 | 1260 | 184.1 | 132.4 | 438.6 | +323.5% | 7.27 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 8168 | 102.7 | 438.7 | 14.1 | 73.3 | 310.7 | -94.6% | 105.5 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2161 | 91.4 | 3516 | 92.4 | 91.9 | 177.9 | +62.7% | 14.7 | **N better** |
| N vs P+ | q6_p50_us | 3605 | 63.2 | 3970 | 61.7 | 62.4 | 710.4 | +10.1% | 5.84 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 129.3 | 7.86 | 659.7 | 69.6 | 49.5 | 97.3 | +410.2% | 10.7 | **N better** |
| N vs P+ | c2_q1_p99_us | 3809 | 500.5 | 5907 | 856.0 | 701.1 | 1281 | +55.1% | 2.99 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 8204 | 166.7 | 561.6 | 62.7 | 126.0 | 2331 | -93.2% | 60.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 133.6 | 16.1 | 1796 | 57.5 | 42.2 | 61.0 | +1243.7% | 39.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 6485 | 667.3 | 8917 | 1576 | 1210 | 3396 | +37.5% | 2.01 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 15735 | 1086 | 1495 | 59.3 | 769.2 | 1403 | -90.5% | 18.5 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 26964 | 2503 | 8264 | 3331 | 2946 | 1979 | -69.4% | 6.35 | **P+ better** |
| N vs P+ | write_p50_us | 613.0 | 41.8 | 795.9 | 30.4 | 36.6 | 82.7 | +29.8% | 5.00 | **N better** |
| N vs P+ | write_p99_us | 970.5 | 159.1 | 2620 | 157.8 | 158.4 | 558.1 | +170.0% | 10.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1517 | 78.6 | 1001 | 88.2 | 83.6 | 308.2 | -34.0% | 6.18 | **N better** |
| N vs P+ | pss_mib | 33.4 | 0.45 | 45.6 | 0.03 | 0.32 | 1.44 | +36.5% | 38.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 24525 | 990.8 | -5761 | 369.9 | 747.8 | 8612 | -123.5% | 40.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 144.8 | 2.61 | 126.0 | 11.7 | 8.49 | 13.0 | -13.0% | 2.21 | no difference |
| N vs P | q1_p99_us | 297.6 | 33.7 | 424.1 | 80.6 | 61.8 | 82.6 | +42.5% | 2.05 | no difference |
| N vs P | q2_p50_us | 8168 | 102.7 | 205.7 | 13.5 | 73.2 | 315.2 | -97.5% | 108.7 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2161 | 91.4 | 149.4 | 8.74 | 64.9 | 145.3 | -93.1% | 31.0 | **P better** |
| N vs P | q6_p50_us | 3605 | 63.2 | 604.9 | 42.8 | 54.0 | 619.4 | -83.2% | 55.6 | **P better** |
| N vs P | c2_q1_p50_us | 129.3 | 7.86 | 124.3 | 12.6 | 10.5 | 40.8 | -3.9% | 0.47 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 3809 | 500.5 | 1616 | 477.8 | 489.3 | 907.9 | -57.6% | 4.48 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 8204 | 166.7 | 219.8 | 28.5 | 119.6 | 2331 | -97.3% | 66.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 133.6 | 16.1 | 250.3 | 16.7 | 16.4 | 137.1 | +87.3% | 7.10 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 6485 | 667.3 | 2569 | 216.6 | 496.1 | 3297 | -60.4% | 7.89 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 15735 | 1086 | 328.2 | 26.6 | 768.2 | 1404 | -97.9% | 20.1 | **P better** |
| N vs P | c4_q2_p99_us | 26964 | 2503 | 2576 | 493.6 | 1804 | 920.1 | -90.4% | 13.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 613.0 | 41.8 | 18040 | 539.5 | 382.6 | 1965 | +2843.1% | 45.5 | **N better** |
| N vs P | write_p99_us | 970.5 | 159.1 | 25798 | 4259 | 3013 | 8801 | +2558.3% | 8.24 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1517 | 78.6 | 51.2 | 3.72 | 55.7 | 306.6 | -96.6% | 26.3 | **N better** |
| N vs P | pss_mib | 33.4 | 0.45 | 45.2 | 0.02 | 0.32 | 1.44 | +35.3% | 37.1 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 24525 | 990.8 | -3503 | 278.4 | 727.7 | 5896 | -114.3% | 38.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 565.0 | 24.4 | 126.0 | 11.7 | 19.1 | 17.8 | -77.7% | 22.9 | **P better** |
| P+ vs P | q1_p99_us | 1260 | 184.1 | 424.1 | 80.6 | 142.1 | 435.1 | -66.3% | 5.88 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 438.7 | 14.1 | 205.7 | 13.5 | 13.8 | 53.6 | -53.1% | 16.9 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 248.7 | 17.7 | 244.0 | 24.7 | 21.5 | 78.2 | -1.9% | 0.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 1516 | 60.8 | 257.7 | 22.3 | 45.8 | 46.0 | -83.0% | 27.5 | **P better** |
| P+ vs P | q5_p50_us | 3516 | 92.4 | 149.4 | 8.74 | 65.7 | 102.6 | -95.8% | 51.3 | **P better** |
| P+ vs P | q6_p50_us | 3970 | 61.7 | 604.9 | 42.8 | 53.1 | 421.0 | -84.8% | 63.4 | **P better** |
| P+ vs P | c2_q1_p50_us | 659.7 | 69.6 | 124.3 | 12.6 | 50.0 | 99.9 | -81.2% | 10.7 | **P better** |
| P+ vs P | c2_q1_p99_us | 5907 | 856.0 | 1616 | 477.8 | 693.2 | 1077 | -72.6% | 6.19 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 561.6 | 62.7 | 219.8 | 28.5 | 48.7 | 68.1 | -60.9% | 7.02 | **P better** |
| P+ vs P | c4_q1_p50_us | 1796 | 57.5 | 250.3 | 16.7 | 42.4 | 150.0 | -86.1% | 36.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8917 | 1576 | 2569 | 216.6 | 1125 | 1180 | -71.2% | 5.64 | **P better** |
| P+ vs P | c4_q2_p50_us | 1495 | 59.3 | 328.2 | 26.6 | 46.0 | 47.8 | -78.0% | 25.4 | **P better** |
| P+ vs P | c4_q2_p99_us | 8264 | 3331 | 2576 | 493.6 | 2381 | 2064 | -68.8% | 2.39 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 795.9 | 30.4 | 18040 | 539.5 | 382.1 | 1965 | +2166.8% | 45.1 | **P+ better** |
| P+ vs P | write_p99_us | 2620 | 157.8 | 25798 | 4259 | 3013 | 8786 | +884.7% | 7.69 | **P+ better** |
| P+ vs P | commits_per_s | 1001 | 88.2 | 51.2 | 3.72 | 62.4 | 35.2 | -94.9% | 15.2 | **P+ better** |
| P+ vs P | pss_mib | 45.6 | 0.03 | 45.2 | 0.02 | 0.02 | 0.07 | -0.8% | 15.1 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -5761 | 369.9 | -3503 | 278.4 | 327.4 | 7397 | -39.2% | 6.90 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 126.0 | 11.7 | 127.5 | 10.00 | 10.9 | 15.5 | +1.2% | 0.14 | BELOW FLOOR |
| P vs M | q1_p99_us | 424.1 | 80.6 | 426.2 | 63.1 | 72.4 | 126.8 | +0.5% | 0.03 | BELOW FLOOR |
| P vs M | q2_p50_us | 205.7 | 13.5 | 206.4 | 20.7 | 17.5 | 53.4 | +0.3% | 0.04 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 244.0 | 24.7 | 253.3 | 30.4 | 27.7 | 78.6 | +3.8% | 0.34 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q4_p50_us | 257.7 | 22.3 | 256.8 | 14.4 | 18.8 | 50.2 | -0.4% | 0.05 | BELOW FLOOR |
| P vs M | q5_p50_us | 149.4 | 8.74 | 164.6 | 13.7 | 11.5 | 43.5 | +10.2% | 1.33 | BELOW FLOOR |
| P vs M | q6_p50_us | 604.9 | 42.8 | 636.1 | 87.4 | 68.8 | 179.0 | +5.2% | 0.45 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 124.3 | 12.6 | 129.4 | 16.6 | 14.7 | 33.2 | +4.1% | 0.35 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1616 | 477.8 | 1808 | 559.5 | 520.3 | 418.7 | +11.9% | 0.37 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 219.8 | 28.5 | 229.7 | 21.1 | 25.1 | 66.3 | +4.5% | 0.39 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 250.3 | 16.7 | 223.8 | 43.1 | 32.7 | 158.3 | -10.6% | 0.81 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2569 | 216.6 | 2110 | 355.3 | 294.2 | 603.8 | -17.9% | 1.56 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 328.2 | 26.6 | 288.6 | 13.5 | 21.1 | 47.7 | -12.1% | 1.88 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2576 | 493.6 | 2083 | 223.3 | 383.1 | 1771 | -19.1% | 1.29 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 18040 | 539.5 | 17849 | 439.5 | 492.0 | 1964 | -1.1% | 0.39 | BELOW FLOOR |
| P vs M | write_p99_us | 25798 | 4259 | 25744 | 4470 | 4366 | 9129 | -0.2% | 0.01 | BELOW FLOOR |
| P vs M | commits_per_s | 51.2 | 3.72 | 53.6 | 1.50 | 2.83 | 11.2 | +4.7% | 0.84 | BELOW FLOOR |
| P vs M | pss_mib | 45.2 | 0.02 | 45.2 | 0.02 | 0.02 | 0.07 | +0.1% | 1.88 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -3503 | 278.4 | -2325 | 149.6 | 223.5 | 3564 | -33.6% | 5.27 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 151.4 | 14.0 | 551.0 | 31.7 | 24.5 | 32.3 | +263.9% | 16.3 | **N better** |
| N vs P+ | q1_p99_us | 314.0 | 60.4 | 1664 | 405.5 | 289.9 | 2669 | +429.9% | 4.66 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 8244 | 194.0 | 423.4 | 25.0 | 138.3 | 1265 | -94.9% | 56.5 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2184 | 74.4 | 3457 | 490.2 | 350.6 | 732.2 | +58.3% | 3.63 | **N better** |
| N vs P+ | q6_p50_us | 3579 | 92.5 | 4122 | 230.5 | 175.6 | 221.7 | +15.2% | 3.09 | **N better** |
| N vs P+ | c2_q1_p50_us | 80.4 | 3.51 | 636.5 | 63.0 | 44.6 | 35.8 | +691.8% | 12.5 | **N better** |
| N vs P+ | c2_q1_p99_us | 339.5 | 83.2 | 5580 | 1753 | 1241 | 2647 | +1543.5% | 4.22 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 8052 | 204.1 | 543.6 | 45.3 | 147.8 | 1708 | -93.2% | 50.8 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 95.0 | 7.35 | 1741 | 80.9 | 57.4 | 363.2 | +1733.3% | 28.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 6540 | 955.9 | 8880 | 411.2 | 735.8 | 3439 | +35.8% | 3.18 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 16070 | 1596 | 1594 | 72.7 | 1130 | 2281 | -90.1% | 12.8 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 35012 | 5297 | 7408 | 698.3 | 3778 | 6824 | -78.8% | 7.31 | **P+ better** |
| N vs P+ | write_p50_us | 605.7 | 40.7 | 779.9 | 42.7 | 41.7 | 55.5 | +28.8% | 4.17 | **N better** |
| N vs P+ | write_p99_us | 934.7 | 175.3 | 2605 | 56.4 | 130.2 | 47.5 | +178.7% | 12.8 | **N better** |
| N vs P+ | commits_per_s | 1552 | 124.6 | 1024 | 11.0 | 88.4 | 108.0 | -34.0% | 5.97 | **N better** |
| N vs P+ | pss_mib | 28.2 | 0.14 | 45.2 | 0.02 | 0.10 | 1.46 | +60.3% | 169.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 20641 | 880.8 | -2473 | 252.8 | 648.0 | 6472 | -112.0% | 35.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 151.4 | 14.0 | 127.2 | 9.47 | 12.0 | 28.0 | -16.0% | 2.03 | BELOW FLOOR |
| N vs P | q1_p99_us | 314.0 | 60.4 | 413.2 | 41.5 | 51.8 | 69.6 | +31.6% | 1.92 | no difference |
| N vs P | q2_p50_us | 8244 | 194.0 | 199.8 | 13.0 | 137.5 | 1263 | -97.6% | 58.5 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2184 | 74.4 | 157.7 | 5.94 | 52.8 | 216.8 | -92.8% | 38.4 | **P better** |
| N vs P | q6_p50_us | 3579 | 92.5 | 652.3 | 36.8 | 70.4 | 220.9 | -81.8% | 41.6 | **P better** |
| N vs P | c2_q1_p50_us | 80.4 | 3.51 | 131.7 | 12.0 | 8.87 | 96.0 | +63.8% | 5.79 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 339.5 | 83.2 | 1764 | 528.7 | 378.4 | 1342 | +419.7% | 3.77 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 8052 | 204.1 | 230.1 | 33.7 | 146.3 | 1713 | -97.1% | 53.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 95.0 | 7.35 | 223.9 | 23.3 | 17.3 | 55.1 | +135.7% | 7.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 6540 | 955.9 | 2295 | 295.0 | 707.4 | 995.9 | -64.9% | 6.00 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 16070 | 1596 | 326.0 | 35.4 | 1129 | 2280 | -98.0% | 13.9 | **P better** |
| N vs P | c4_q2_p99_us | 35012 | 5297 | 2620 | 227.7 | 3749 | 6815 | -92.5% | 8.64 | **P better** |
| N vs P | write_p50_us | 605.7 | 40.7 | 18094 | 367.9 | 261.8 | 549.6 | +2887.4% | 66.8 | **N better** |
| N vs P | write_p99_us | 934.7 | 175.3 | 26068 | 3186 | 2256 | 3989 | +2689.1% | 11.1 | **N better** |
| N vs P | commits_per_s | 1552 | 124.6 | 52.5 | 1.48 | 88.1 | 82.6 | -96.6% | 17.0 | **N better** |
| N vs P | pss_mib | 28.2 | 0.14 | 45.2 | 0.01 | 0.10 | 1.46 | +60.3% | 170.3 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 20641 | 880.8 | -2197 | 131.8 | 629.8 | 6293 | -110.6% | 36.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 551.0 | 31.7 | 127.2 | 9.47 | 23.4 | 39.3 | -76.9% | 18.1 | **P better** |
| P+ vs P | q1_p99_us | 1664 | 405.5 | 413.2 | 41.5 | 288.2 | 2668 | -75.2% | 4.34 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 423.4 | 25.0 | 199.8 | 13.0 | 19.9 | 62.5 | -52.8% | 11.2 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 237.1 | 17.8 | 236.2 | 20.5 | 19.2 | 53.0 | -0.4% | 0.04 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1480 | 45.5 | 252.8 | 12.0 | 33.3 | 35.6 | -82.9% | 36.9 | **P better** |
| P+ vs P | q5_p50_us | 3457 | 490.2 | 157.7 | 5.94 | 346.6 | 699.3 | -95.4% | 9.52 | **P better** |
| P+ vs P | q6_p50_us | 4122 | 230.5 | 652.3 | 36.8 | 165.0 | 80.8 | -84.2% | 21.0 | **P better** |
| P+ vs P | c2_q1_p50_us | 636.5 | 63.0 | 131.7 | 12.0 | 45.4 | 99.6 | -79.3% | 11.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5580 | 1753 | 1764 | 528.7 | 1295 | 2308 | -68.4% | 2.95 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 543.6 | 45.3 | 230.1 | 33.7 | 39.9 | 129.7 | -57.7% | 7.85 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1741 | 80.9 | 223.9 | 23.3 | 59.5 | 363.9 | -87.1% | 25.5 | **P better** |
| P+ vs P | c4_q1_p99_us | 8880 | 411.2 | 2295 | 295.0 | 357.9 | 3547 | -74.2% | 18.4 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1594 | 72.7 | 326.0 | 35.4 | 57.2 | 84.0 | -79.6% | 22.2 | **P better** |
| P+ vs P | c4_q2_p99_us | 7408 | 698.3 | 2620 | 227.7 | 519.4 | 644.8 | -64.6% | 9.22 | **P better** |
| P+ vs P | write_p50_us | 779.9 | 42.7 | 18094 | 367.9 | 261.9 | 551.8 | +2220.0% | 66.1 | **P+ better** |
| P+ vs P | write_p99_us | 2605 | 56.4 | 26068 | 3186 | 2253 | 3988 | +900.7% | 10.4 | **P+ better** |
| P+ vs P | commits_per_s | 1024 | 11.0 | 52.5 | 1.48 | 7.82 | 69.6 | -94.9% | 124.1 | **P+ better** |
| P+ vs P | pss_mib | 45.2 | 0.02 | 45.2 | 0.01 | 0.02 | 0.01 | +0.0% | 0.00 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2473 | 252.8 | -2197 | 131.8 | 201.6 | 3074 | -11.2% | 1.37 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 127.2 | 9.47 | 128.5 | 5.75 | 7.83 | 27.0 | +1.0% | 0.17 | BELOW FLOOR |
| P vs M | q1_p99_us | 413.2 | 41.5 | 421.5 | 86.4 | 67.8 | 78.1 | +2.0% | 0.12 | BELOW FLOOR |
| P vs M | q2_p50_us | 199.8 | 13.0 | 196.5 | 7.37 | 10.5 | 16.2 | -1.7% | 0.31 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 236.2 | 20.5 | 229.9 | 30.3 | 25.9 | 85.7 | -2.7% | 0.24 | BELOW FLOOR |
| P vs M | q4_p50_us | 252.8 | 12.0 | 260.6 | 25.5 | 19.9 | 23.9 | +3.1% | 0.39 | BELOW FLOOR |
| P vs M | q5_p50_us | 157.7 | 5.94 | 168.2 | 15.9 | 12.0 | 29.4 | +6.7% | 0.88 | BELOW FLOOR |
| P vs M | q6_p50_us | 652.3 | 36.8 | 607.9 | 55.1 | 46.9 | 55.8 | -6.8% | 0.95 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 131.7 | 12.0 | 124.8 | 21.1 | 17.2 | 95.3 | -5.2% | 0.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1764 | 528.7 | 1988 | 493.4 | 511.4 | 282.3 | +12.7% | 0.44 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 230.1 | 33.7 | 204.5 | 34.2 | 34.0 | 133.5 | -11.1% | 0.76 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 223.9 | 23.3 | 259.3 | 32.1 | 28.1 | 57.5 | +15.8% | 1.26 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2295 | 295.0 | 2709 | 474.2 | 394.9 | 961.5 | +18.0% | 1.05 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 326.0 | 35.4 | 327.9 | 21.7 | 29.4 | 55.1 | +0.6% | 0.07 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2620 | 227.7 | 3146 | 673.8 | 502.9 | 4233 | +20.1% | 1.05 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 18094 | 367.9 | 17738 | 160.5 | 283.8 | 778.9 | -2.0% | 1.25 | BELOW FLOOR |
| P vs M | write_p99_us | 26068 | 3186 | 23267 | 2041 | 2676 | 3989 | -10.7% | 1.05 | BELOW FLOOR |
| P vs M | commits_per_s | 52.5 | 1.48 | 53.8 | 2.22 | 1.89 | 1.38 | +2.5% | 0.69 | BELOW FLOOR |
| P vs M | pss_mib | 45.2 | 0.01 | 45.6 | 0.05 | 0.04 | 0.12 | +0.9% | 11.4 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2197 | 131.8 | -2214 | 123.5 | 127.7 | 3768 | +0.8% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 144.4 | 7.24 | 527.9 | 21.4 | 16.0 | 43.1 | +265.7% | 24.0 | **N better** |
| N vs P+ | q1_p99_us | 325.7 | 29.8 | 1220 | 233.9 | 166.7 | 2140 | +274.6% | 5.36 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 8223 | 142.8 | 406.7 | 25.1 | 102.5 | 1930 | -95.1% | 76.3 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 1746 | 525.7 | 2806 | 905.6 | 740.5 | 630.7 | +60.7% | 1.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q6_p50_us | 3537 | 107.6 | 4058 | 235.5 | 183.1 | 230.2 | +14.7% | 2.84 | no difference |
| N vs P+ | c2_q1_p50_us | 92.3 | 15.0 | 594.3 | 39.2 | 29.7 | 78.2 | +543.9% | 16.9 | **N better** |
| N vs P+ | c2_q1_p99_us | 291.3 | 24.2 | 5099 | 545.4 | 386.0 | 683.1 | +1650.6% | 12.5 | **N better** |
| N vs P+ | c2_q2_p50_us | 8365 | 226.7 | 491.7 | 54.4 | 164.9 | 1521 | -94.1% | 47.8 | **P+ better** |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c4_q1_p50_us | 89.5 | 8.36 | 1639 | 63.3 | 45.1 | 89.2 | +1731.4% | 34.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 5764 | 598.7 | 9508 | 2986 | 2154 | 15976 | +65.0% | 1.74 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 16758 | 168.3 | 1383 | 132.3 | 151.4 | 4603 | -91.7% | 101.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 29991 | 3813 | 7834 | 2401 | 3186 | 8674 | -73.9% | 6.95 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 626.0 | 30.8 | 788.5 | 51.3 | 42.3 | 83.2 | +26.0% | 3.84 | **N better** |
| N vs P+ | write_p99_us | 1231 | 307.4 | 2506 | 154.0 | 243.1 | 183.3 | +103.5% | 5.24 | **N better** |
| N vs P+ | commits_per_s | 1542 | 101.9 | 1073 | 68.6 | 86.8 | 189.8 | -30.4% | 5.40 | **N better** |
| N vs P+ | pss_mib | 24.9 | 2.83 | 45.1 | 0.02 | 2.00 | 1.26 | +81.4% | 10.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 19940 | 2874 | -2314 | 219.2 | 2038 | 4388 | -111.6% | 10.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 144.4 | 7.24 | 117.4 | 4.10 | 5.88 | 7.96 | -18.6% | 4.57 | **P better** |
| N vs P | q1_p99_us | 325.7 | 29.8 | 381.2 | 38.0 | 34.1 | 414.0 | +17.1% | 1.63 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 8223 | 142.8 | 195.7 | 7.14 | 101.1 | 1930 | -97.6% | 79.4 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 1746 | 525.7 | 149.4 | 3.70 | 371.8 | 160.3 | -91.4% | 4.30 | **P better** |
| N vs P | q6_p50_us | 3537 | 107.6 | 548.8 | 47.3 | 83.1 | 56.7 | -84.5% | 35.9 | **P better** |
| N vs P | c2_q1_p50_us | 92.3 | 15.0 | 117.9 | 9.37 | 12.5 | 26.9 | +27.8% | 2.05 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 291.3 | 24.2 | 1403 | 268.9 | 190.9 | 1497 | +381.8% | 5.82 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 8365 | 226.7 | 200.1 | 24.4 | 161.3 | 1521 | -97.6% | 50.6 | **P better** |
| N vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c4_q1_p50_us | 89.5 | 8.36 | 206.9 | 43.9 | 31.6 | 32.1 | +131.1% | 3.71 | **N better** |
| N vs P | c4_q1_p99_us | 5764 | 598.7 | 2514 | 321.7 | 480.6 | 577.0 | -56.4% | 6.76 | **P better** |
| N vs P | c4_q2_p50_us | 16758 | 168.3 | 286.8 | 37.2 | 121.9 | 4600 | -98.3% | 135.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 29991 | 3813 | 2840 | 244.6 | 2702 | 5930 | -90.5% | 10.0 | **P better** |
| N vs P | write_p50_us | 626.0 | 30.8 | 18005 | 294.1 | 209.1 | 612.3 | +2776.0% | 83.1 | **N better** |
| N vs P | write_p99_us | 1231 | 307.4 | 24543 | 2842 | 2021 | 6680 | +1893.0% | 11.5 | **N better** |
| N vs P | commits_per_s | 1542 | 101.9 | 52.6 | 1.97 | 72.1 | 33.8 | -96.6% | 20.7 | **N better** |
| N vs P | pss_mib | 24.9 | 2.83 | 45.6 | 0.04 | 2.00 | 1.26 | +83.4% | 10.3 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 19940 | 2874 | -2021 | 172.6 | 2036 | 3851 | -110.1% | 10.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 527.9 | 21.4 | 117.4 | 4.10 | 15.4 | 42.4 | -77.8% | 26.6 | **P better** |
| P+ vs P | q1_p99_us | 1220 | 233.9 | 381.2 | 38.0 | 167.5 | 2178 | -68.7% | 5.01 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 406.7 | 25.1 | 195.7 | 7.14 | 18.5 | 1.80 | -51.9% | 11.4 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 238.8 | 41.0 | 225.1 | 26.4 | 34.5 | 43.2 | -5.7% | 0.40 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1470 | 36.1 | 243.3 | 15.1 | 27.7 | 82.7 | -83.4% | 44.3 | **P better** |
| P+ vs P | q5_p50_us | 2806 | 905.6 | 149.4 | 3.70 | 640.4 | 610.7 | -94.7% | 4.15 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q6_p50_us | 4058 | 235.5 | 548.8 | 47.3 | 169.8 | 233.7 | -86.5% | 20.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 594.3 | 39.2 | 117.9 | 9.37 | 28.5 | 75.0 | -80.2% | 16.7 | **P better** |
| P+ vs P | c2_q1_p99_us | 5099 | 545.4 | 1403 | 268.9 | 430.0 | 1642 | -72.5% | 8.60 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 491.7 | 54.4 | 200.1 | 24.4 | 42.1 | 27.9 | -59.3% | 6.92 | **P better** |
| P+ vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c4_q1_p50_us | 1639 | 63.3 | 206.9 | 43.9 | 54.5 | 83.4 | -87.4% | 26.3 | **P better** |
| P+ vs P | c4_q1_p99_us | 9508 | 2986 | 2514 | 321.7 | 2124 | 15986 | -73.6% | 3.29 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1383 | 132.3 | 286.8 | 37.2 | 97.2 | 179.1 | -79.3% | 11.3 | **P better** |
| P+ vs P | c4_q2_p99_us | 7834 | 2401 | 2840 | 244.6 | 1707 | 6342 | -63.7% | 2.93 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 788.5 | 51.3 | 18005 | 294.1 | 211.1 | 617.5 | +2183.4% | 81.6 | **P+ better** |
| P+ vs P | write_p99_us | 2506 | 154.0 | 24543 | 2842 | 2012 | 6682 | +879.6% | 11.0 | **P+ better** |
| P+ vs P | commits_per_s | 1073 | 68.6 | 52.6 | 1.97 | 48.5 | 186.8 | -95.1% | 21.0 | **P+ better** |
| P+ vs P | pss_mib | 45.1 | 0.02 | 45.6 | 0.04 | 0.03 | 0.07 | +1.1% | 17.4 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2314 | 219.2 | -2021 | 172.6 | 197.3 | 3355 | -12.6% | 1.48 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 117.4 | 4.10 | 126.6 | 11.4 | 8.57 | 14.9 | +7.8% | 1.07 | BELOW FLOOR |
| P vs M | q1_p99_us | 381.2 | 38.0 | 543.3 | 187.1 | 135.0 | 583.4 | +42.5% | 1.20 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 195.7 | 7.14 | 210.4 | 17.1 | 13.1 | 12.0 | +7.5% | 1.12 | no difference |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 225.1 | 26.4 | 268.3 | 29.4 | 28.0 | 38.1 | +19.2% | 1.55 | no difference |
| P vs M | q4_p50_us | 243.3 | 15.1 | 254.5 | 12.7 | 14.0 | 32.9 | +4.6% | 0.81 | BELOW FLOOR |
| P vs M | q5_p50_us | 149.4 | 3.70 | 164.3 | 9.69 | 7.34 | 22.0 | +10.0% | 2.03 | BELOW FLOOR |
| P vs M | q6_p50_us | 548.8 | 47.3 | 640.2 | 65.4 | 57.0 | 50.0 | +16.7% | 1.60 | no difference |
| P vs M | c2_q1_p50_us | 117.9 | 9.37 | 143.1 | 23.3 | 17.7 | 94.4 | +21.4% | 1.42 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1403 | 268.9 | 2150 | 250.5 | 259.9 | 1794 | +53.2% | 2.87 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 200.1 | 24.4 | 230.9 | 44.4 | 35.8 | 106.8 | +15.4% | 0.86 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c4_q1_p50_us | 206.9 | 43.9 | 217.2 | 45.1 | 44.5 | 13.0 | +5.0% | 0.23 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2514 | 321.7 | 2579 | 316.6 | 319.1 | 575.3 | +2.6% | 0.20 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 286.8 | 37.2 | 300.9 | 33.5 | 35.4 | 50.3 | +4.9% | 0.40 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2840 | 244.6 | 3262 | 425.0 | 346.8 | 694.6 | +14.9% | 1.22 | BELOW FLOOR |
| P vs M | write_p50_us | 18005 | 294.1 | 18320 | 706.4 | 541.0 | 614.2 | +1.7% | 0.58 | BELOW FLOOR |
| P vs M | write_p99_us | 24543 | 2842 | 27681 | 5034 | 4088 | 7294 | +12.8% | 0.77 | BELOW FLOOR |
| P vs M | commits_per_s | 52.6 | 1.97 | 52.3 | 3.74 | 2.99 | 3.57 | -0.5% | 0.08 | BELOW FLOOR |
| P vs M | pss_mib | 45.6 | 0.04 | 45.2 | 0.05 | 0.05 | 0.06 | -0.9% | 9.36 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2021 | 172.6 | -2256 | 156.7 | 164.8 | 2832 | +11.6% | 1.42 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 149.3 | 6.95 | 535.3 | 8.81 | 7.93 | 32.8 | +258.6% | 48.7 | **N better** |
| N vs P+ | q1_p99_us | 319.1 | 59.6 | 1117 | 200.1 | 147.7 | 842.7 | +250.1% | 5.41 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 8285 | 391.9 | 413.5 | 15.9 | 277.4 | 251.5 | -95.0% | 28.4 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2182 | 59.1 | 3566 | 104.3 | 84.8 | 1756 | +63.4% | 16.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q6_p50_us | 3502 | 138.4 | 4052 | 73.8 | 110.9 | 180.2 | +15.7% | 4.96 | **N better** |
| N vs P+ | c2_q1_p50_us | 90.3 | 13.8 | 625.0 | 15.4 | 14.6 | 25.0 | +592.5% | 36.6 | **N better** |
| N vs P+ | c2_q1_p99_us | 335.0 | 58.7 | 5376 | 1311 | 928.0 | 1501 | +1504.7% | 5.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 8148 | 134.5 | 526.2 | 16.6 | 95.8 | 281.9 | -93.5% | 79.6 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 87.3 | 6.01 | 1695 | 81.4 | 57.7 | 77.4 | +1841.7% | 27.8 | **N better** |
| N vs P+ | c4_q1_p99_us | 5456 | 295.1 | 8259 | 1414 | 1021 | 3775 | +51.4% | 2.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 16381 | 728.9 | 1469 | 90.3 | 519.4 | 1828 | -91.0% | 28.7 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 30736 | 2609 | 6854 | 1677 | 2193 | 6963 | -77.7% | 10.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 626.2 | 40.8 | 768.9 | 29.9 | 35.8 | 129.3 | +22.8% | 3.99 | **N better** |
| N vs P+ | write_p99_us | 895.5 | 166.5 | 2563 | 171.4 | 168.9 | 746.1 | +186.2% | 9.87 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1573 | 130.3 | 1126 | 35.7 | 95.5 | 137.7 | -28.4% | 4.68 | **N better** |
| N vs P+ | pss_mib | 28.3 | 0.28 | 45.2 | 0.05 | 0.20 | 0.79 | +59.6% | 83.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 20012 | 506.7 | -2228 | 232.4 | 394.2 | 2354 | -111.1% | 56.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 149.3 | 6.95 | 125.0 | 12.0 | 9.80 | 22.1 | -16.3% | 2.48 | no difference |
| N vs P | q1_p99_us | 319.1 | 59.6 | 359.5 | 45.5 | 53.0 | 28.4 | +12.7% | 0.76 | no difference |
| N vs P | q2_p50_us | 8285 | 391.9 | 209.5 | 11.6 | 277.3 | 251.1 | -97.5% | 29.1 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2182 | 59.1 | 158.8 | 7.98 | 42.2 | 1572 | -92.7% | 47.9 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q6_p50_us | 3502 | 138.4 | 535.4 | 30.5 | 100.2 | 178.6 | -84.7% | 29.6 | **P better** |
| N vs P | c2_q1_p50_us | 90.3 | 13.8 | 128.3 | 13.2 | 13.5 | 38.9 | +42.2% | 2.82 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 335.0 | 58.7 | 1632 | 462.6 | 329.8 | 834.2 | +387.1% | 3.93 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 8148 | 134.5 | 212.7 | 3.73 | 95.1 | 283.0 | -97.4% | 83.4 | **P better** |
| N vs P | c4_q1_p50_us | 87.3 | 6.01 | 226.3 | 25.4 | 18.5 | 28.7 | +159.2% | 7.52 | **N better** |
| N vs P | c4_q1_p99_us | 5456 | 295.1 | 2530 | 234.4 | 266.5 | 3850 | -53.6% | 11.0 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p50_us | 16381 | 728.9 | 309.8 | 14.7 | 515.5 | 1788 | -98.1% | 31.2 | **P better** |
| N vs P | c4_q2_p99_us | 30736 | 2609 | 2797 | 393.9 | 1866 | 6423 | -90.9% | 15.0 | **P better** |
| N vs P | write_p50_us | 626.2 | 40.8 | 17741 | 158.5 | 115.7 | 431.7 | +2733.0% | 147.9 | **N better** |
| N vs P | write_p99_us | 895.5 | 166.5 | 25999 | 4451 | 3149 | 1278 | +2803.2% | 7.97 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1573 | 130.3 | 53.0 | 1.40 | 92.1 | 115.5 | -96.6% | 16.5 | **N better** |
| N vs P | pss_mib | 28.3 | 0.28 | 45.4 | 0.01 | 0.20 | 0.79 | +60.4% | 85.3 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 20012 | 506.7 | -2103 | 138.2 | 371.4 | 1587 | -110.5% | 59.6 | **P better** |
| P+ vs P | q1_p50_us | 535.3 | 8.81 | 125.0 | 12.0 | 10.5 | 24.6 | -76.6% | 39.0 | **P better** |
| P+ vs P | q1_p99_us | 1117 | 200.1 | 359.5 | 45.5 | 145.1 | 842.8 | -67.8% | 5.22 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 413.5 | 15.9 | 209.5 | 11.6 | 13.9 | 26.9 | -49.3% | 14.6 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 227.9 | 14.0 | 248.3 | 24.7 | 20.1 | 105.2 | +9.0% | 1.01 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q4_p50_us | 1477 | 51.4 | 265.3 | 17.5 | 38.4 | 165.1 | -82.0% | 31.6 | **P better** |
| P+ vs P | q5_p50_us | 3566 | 104.3 | 158.8 | 7.98 | 74.0 | 849.8 | -95.5% | 46.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 4052 | 73.8 | 535.4 | 30.5 | 56.4 | 53.3 | -86.8% | 62.3 | **P better** |
| P+ vs P | c2_q1_p50_us | 625.0 | 15.4 | 128.3 | 13.2 | 14.4 | 31.7 | -79.5% | 34.6 | **P better** |
| P+ vs P | c2_q1_p99_us | 5376 | 1311 | 1632 | 462.6 | 983.1 | 1652 | -69.6% | 3.81 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 526.2 | 16.6 | 212.7 | 3.73 | 12.0 | 26.2 | -59.6% | 26.1 | **P better** |
| P+ vs P | c4_q1_p50_us | 1695 | 81.4 | 226.3 | 25.4 | 60.3 | 74.0 | -86.6% | 24.3 | **P better** |
| P+ vs P | c4_q1_p99_us | 8259 | 1414 | 2530 | 234.4 | 1013 | 881.5 | -69.4% | 5.65 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1469 | 90.3 | 309.8 | 14.7 | 64.7 | 380.0 | -78.9% | 17.9 | **P better** |
| P+ vs P | c4_q2_p99_us | 6854 | 1677 | 2797 | 393.9 | 1218 | 2786 | -59.2% | 3.33 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 768.9 | 29.9 | 17741 | 158.5 | 114.0 | 450.5 | +2207.4% | 148.8 | **P+ better** |
| P+ vs P | write_p99_us | 2563 | 171.4 | 25999 | 4451 | 3149 | 1108 | +914.3% | 7.44 | **P+ better** |
| P+ vs P | commits_per_s | 1126 | 35.7 | 53.0 | 1.40 | 25.2 | 75.1 | -95.3% | 42.5 | **P+ better** |
| P+ vs P | pss_mib | 45.2 | 0.05 | 45.4 | 0.01 | 0.03 | 0.10 | +0.5% | 6.26 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2228 | 232.4 | -2103 | 138.2 | 191.2 | 2188 | -5.6% | 0.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P vs M | q1_p50_us | 125.0 | 12.0 | 127.3 | 3.74 | 8.89 | 8.51 | +1.8% | 0.26 | BELOW FLOOR |
| P vs M | q1_p99_us | 359.5 | 45.5 | 378.5 | 44.1 | 44.8 | 144.3 | +5.3% | 0.42 | BELOW FLOOR |
| P vs M | q2_p50_us | 209.5 | 11.6 | 207.3 | 16.4 | 14.2 | 35.0 | -1.1% | 0.16 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 248.3 | 24.7 | 225.6 | 10.1 | 18.9 | 84.8 | -9.1% | 1.20 | BELOW FLOOR |
| P vs M | q4_p50_us | 265.3 | 17.5 | 260.7 | 11.6 | 14.8 | 74.0 | -1.7% | 0.31 | BELOW FLOOR |
| P vs M | q5_p50_us | 158.8 | 7.98 | 153.8 | 11.8 | 10.1 | 258.9 | -3.1% | 0.50 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | 535.4 | 30.5 | 579.2 | 61.1 | 48.3 | 38.9 | +8.2% | 0.91 | no difference |
| P vs M | c2_q1_p50_us | 128.3 | 13.2 | 150.1 | 28.1 | 22.0 | 39.4 | +17.0% | 0.99 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1632 | 462.6 | 2002 | 142.9 | 342.4 | 984.1 | +22.7% | 1.08 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 212.7 | 3.73 | 230.3 | 34.2 | 24.3 | 49.5 | +8.3% | 0.72 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 226.3 | 25.4 | 230.8 | 33.6 | 29.8 | 52.4 | +2.0% | 0.15 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2530 | 234.4 | 2347 | 456.6 | 363.0 | 918.5 | -7.2% | 0.50 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 309.8 | 14.7 | 308.0 | 23.3 | 19.5 | 49.8 | -0.6% | 0.09 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2797 | 393.9 | 2592 | 378.4 | 386.3 | 840.6 | -7.3% | 0.53 | BELOW FLOOR |
| P vs M | write_p50_us | 17741 | 158.5 | 17710 | 347.9 | 270.3 | 686.0 | -0.2% | 0.12 | BELOW FLOOR |
| P vs M | write_p99_us | 25999 | 4451 | 27769 | 3258 | 3900 | 9497 | +6.8% | 0.45 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | commits_per_s | 53.0 | 1.40 | 53.8 | 2.37 | 1.94 | 2.93 | +1.5% | 0.40 | BELOW FLOOR |
| P vs M | pss_mib | 45.4 | 0.01 | 45.2 | 0.01 | 0.01 | 0.10 | -0.5% | 16.1 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2103 | 138.2 | -2180 | 139.9 | 139.1 | 2205 | +3.6% | 0.55 | REFUSED (warm-up MAD above 15% of the median on M) |

## `multi`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 143.7 | 5.54 | 519.5 | 21.4 | 15.6 | 27.8 | +261.5% | 24.1 | **N better** |
| N vs P+ | q1_p99_us | 323.7 | 58.9 | 1620 | 347.9 | 249.5 | 147.3 | +400.4% | 5.19 | **N better** |
| N vs P+ | q2_p50_us | 8008 | 80.3 | 411.0 | 15.1 | 57.7 | 62.0 | -94.9% | 131.6 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2035 | 340.1 | 3119 | 475.5 | 413.4 | 579.3 | +53.2% | 2.62 | no difference |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 96.6 | 17.6 | 557.9 | 26.3 | 22.4 | 4.93 | +477.5% | 20.6 | **N better** |
| N vs P+ | c2_q1_p99_us | 314.1 | 104.0 | 5143 | 429.9 | 312.8 | 3056 | +1537.4% | 15.4 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 7978 | 67.0 | 488.5 | 24.2 | 50.4 | 537.7 | -93.9% | 148.6 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 89.0 | 3.30 | 1507 | 32.1 | 22.8 | 204.3 | +1592.8% | 62.2 | **N better** |
| N vs P+ | c4_q1_p99_us | 5669 | 765.1 | 7856 | 1889 | 1441 | 1910 | +38.6% | 1.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 15613 | 604.8 | 1291 | 50.5 | 429.2 | 1028 | -91.7% | 33.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 29679 | 3118 | 8596 | 3688 | 3415 | 4429 | -71.0% | 6.17 | **P+ better** |
| N vs P+ | write_p50_us | 533.1 | 39.8 | 740.5 | 36.6 | 38.2 | 219.0 | +38.9% | 5.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 840.6 | 126.6 | 2485 | 97.7 | 113.0 | 262.9 | +195.6% | 14.5 | **N better** |
| N vs P+ | commits_per_s | 1771 | 193.4 | 1174 | 26.0 | 138.0 | 506.2 | -33.7% | 4.33 | **N better** |
| N vs P+ | pss_mib | 28.3 | 0.47 | 45.4 | 0.04 | 0.34 | 0.32 | +60.4% | 51.0 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 20094 | 1106 | -2492 | 230.6 | 798.8 | 5091 | -112.4% | 28.3 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 143.7 | 5.54 | 117.0 | 5.29 | 5.42 | 24.1 | -18.6% | 4.94 | **P better** |
| N vs P | q1_p99_us | 323.7 | 58.9 | 333.2 | 58.5 | 58.7 | 307.6 | +2.9% | 0.16 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 8008 | 80.3 | 179.8 | 5.15 | 56.9 | 68.7 | -97.8% | 137.7 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2035 | 340.1 | 140.7 | 8.44 | 240.5 | 97.5 | -93.1% | 7.88 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 96.6 | 17.6 | 88.8 | 39.7 | 30.7 | 7.72 | -8.1% | 0.25 | no difference |
| N vs P | c2_q1_p99_us | 314.1 | 104.0 | 1108 | 150.2 | 129.2 | 1727 | +252.6% | 6.14 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 7978 | 67.0 | 150.9 | 58.5 | 62.9 | 538.7 | -98.1% | 124.4 | **P better** |
| N vs P | c4_q1_p50_us | 89.0 | 3.30 | 163.7 | 34.1 | 24.2 | 63.8 | +83.9% | 3.09 | **N better** |
| N vs P | c4_q1_p99_us | 5669 | 765.1 | 1930 | 328.4 | 588.7 | 1912 | -66.0% | 6.35 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 15613 | 604.8 | 260.2 | 39.7 | 428.6 | 998.6 | -98.3% | 35.8 | **P better** |
| N vs P | c4_q2_p99_us | 29679 | 3118 | 2178 | 340.0 | 2218 | 4339 | -92.7% | 12.4 | **P better** |
| N vs P | write_p50_us | 533.1 | 39.8 | 17023 | 175.3 | 127.1 | 215.0 | +3093.5% | 129.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p99_us | 840.6 | 126.6 | 21125 | 376.2 | 280.6 | 6809 | +2413.2% | 72.3 | **N better** |
| N vs P | commits_per_s | 1771 | 193.4 | 57.5 | 0.91 | 136.8 | 398.4 | -96.8% | 12.5 | **N better** |
| N vs P | pss_mib | 28.3 | 0.47 | 45.4 | 0.01 | 0.33 | 0.65 | +60.5% | 51.2 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 20094 | 1106 | -2309 | 161.0 | 790.2 | 6351 | -111.5% | 28.4 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 519.5 | 21.4 | 117.0 | 5.29 | 15.6 | 30.0 | -77.5% | 25.8 | **P better** |
| P+ vs P | q1_p99_us | 1620 | 347.9 | 333.2 | 58.5 | 249.4 | 330.1 | -79.4% | 5.16 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 411.0 | 15.1 | 179.8 | 5.15 | 11.3 | 33.2 | -56.3% | 20.5 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 214.5 | 11.6 | 214.7 | 19.0 | 15.8 | 54.5 | +0.1% | 0.01 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1443 | 19.9 | 249.3 | 16.1 | 18.1 | 34.3 | -82.7% | 65.8 | **P better** |
| P+ vs P | q5_p50_us | 3119 | 475.5 | 140.7 | 8.44 | 336.3 | 571.5 | -95.5% | 8.86 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 557.9 | 26.3 | 88.8 | 39.7 | 33.7 | 7.63 | -84.1% | 13.9 | **P better** |
| P+ vs P | c2_q1_p99_us | 5143 | 429.9 | 1108 | 150.2 | 322.0 | 3137 | -78.5% | 12.5 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 488.5 | 24.2 | 150.9 | 58.5 | 44.8 | 43.5 | -69.1% | 7.54 | **P better** |
| P+ vs P | c4_q1_p50_us | 1507 | 32.1 | 163.7 | 34.1 | 33.1 | 213.9 | -89.1% | 40.6 | **P better** |
| P+ vs P | c4_q1_p99_us | 7856 | 1889 | 1930 | 328.4 | 1356 | 231.5 | -75.4% | 4.37 | **P better** |
| P+ vs P | c4_q2_p50_us | 1291 | 50.5 | 260.2 | 39.7 | 45.4 | 248.6 | -79.9% | 22.7 | **P better** |
| P+ vs P | c4_q2_p99_us | 8596 | 3688 | 2178 | 340.0 | 2619 | 1243 | -74.7% | 2.45 | no difference |
| P+ vs P | write_p50_us | 740.5 | 36.6 | 17023 | 175.3 | 126.7 | 81.1 | +2198.8% | 128.6 | **P+ better** |
| P+ vs P | write_p99_us | 2485 | 97.7 | 21125 | 376.2 | 274.8 | 6814 | +750.3% | 67.8 | **P+ better** |
| P+ vs P | commits_per_s | 1174 | 26.0 | 57.5 | 0.91 | 18.4 | 312.3 | -95.1% | 60.7 | **P+ better** |
| P+ vs P | pss_mib | 45.4 | 0.04 | 45.4 | 0.01 | 0.03 | 0.57 | +0.0% | 0.48 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2492 | 230.6 | -2309 | 161.0 | 198.9 | 4878 | -7.3% | 0.92 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 117.0 | 5.29 | 117.8 | 2.60 | 4.17 | 20.6 | +0.7% | 0.21 | BELOW FLOOR |
| P vs M | q1_p99_us | 333.2 | 58.5 | 355.6 | 41.2 | 50.6 | 411.3 | +6.7% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 179.8 | 5.15 | 181.2 | 7.71 | 6.56 | 32.6 | +0.8% | 0.21 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 214.7 | 19.0 | 204.0 | 20.0 | 19.5 | 34.4 | -5.0% | 0.55 | BELOW FLOOR |
| P vs M | q4_p50_us | 249.3 | 16.1 | 255.6 | 18.2 | 17.2 | 17.4 | +2.5% | 0.37 | BELOW FLOOR |
| P vs M | q5_p50_us | 140.7 | 8.44 | 144.2 | 8.09 | 8.26 | 17.4 | +2.5% | 0.43 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 88.8 | 39.7 | 130.3 | 7.79 | 28.6 | 15.2 | +46.8% | 1.45 | no difference |
| P vs M | c2_q1_p99_us | 1108 | 150.2 | 1349 | 354.9 | 272.5 | 2365 | +21.8% | 0.89 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 150.9 | 58.5 | 226.7 | 7.83 | 41.7 | 86.2 | +50.2% | 1.81 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 163.7 | 34.1 | 206.3 | 47.9 | 41.6 | 86.5 | +26.0% | 1.02 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 1930 | 328.4 | 2437 | 199.7 | 271.8 | 365.6 | +26.3% | 1.87 | no difference |
| P vs M | c4_q2_p50_us | 260.2 | 39.7 | 293.5 | 37.0 | 38.4 | 125.1 | +12.8% | 0.87 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p99_us | 2178 | 340.0 | 2773 | 405.6 | 374.3 | 615.0 | +27.3% | 1.59 | BELOW FLOOR |
| P vs M | write_p50_us | 17023 | 175.3 | 17113 | 295.4 | 242.9 | 528.1 | +0.5% | 0.37 | BELOW FLOOR |
| P vs M | write_p99_us | 21125 | 376.2 | 22682 | 1638 | 1189 | 6972 | +7.4% | 1.31 | BELOW FLOOR |
| P vs M | commits_per_s | 57.5 | 0.91 | 56.8 | 0.45 | 0.72 | 0.75 | -1.3% | 1.05 | BELOW FLOOR |
| P vs M | pss_mib | 45.4 | 0.01 | 45.2 | 0.02 | 0.02 | 0.57 | -0.5% | 12.9 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2309 | 161.0 | -2300 | 204.0 | 183.8 | 4858 | -0.4% | 0.05 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 173.4 | 7.35 | 579.3 | 35.2 | 25.4 | 74.3 | +234.0% | 16.0 | **N better** |
| N vs P+ | q1_p99_us | 414.6 | 29.9 | 1934 | 530.6 | 375.8 | 2963 | +366.5% | 4.04 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 73700 | 5324 | 458.4 | 29.1 | 3765 | 4713 | -99.4% | 19.5 | **P+ better** |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 10539 | 2531 | 17437 | 2457 | 2494 | 4708 | +65.5% | 2.77 | no difference |
| N vs P+ | q6_p50_us | 50023 | 1170 | 46092 | 2154 | 1733 | 7788 | -7.9% | 2.27 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 129.9 | 17.9 | 621.8 | 62.6 | 46.0 | 145.0 | +378.7% | 10.7 | **N better** |
| N vs P+ | c2_q1_p99_us | 480.0 | 152.2 | 5225 | 289.3 | 231.1 | 2502 | +988.6% | 20.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 82081 | 9149 | 525.5 | 49.2 | 6469 | 1126 | -99.4% | 12.6 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 144.6 | 9.48 | 1779 | 96.9 | 68.8 | 83.1 | +1130.3% | 23.8 | **N better** |
| N vs P+ | c4_q1_p99_us | 8022 | 493.8 | 7845 | 1314 | 992.6 | 2020 | -2.2% | 0.18 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 121680 | 10435 | 1545 | 157.9 | 7380 | 1518 | -98.7% | 16.3 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 262616 | 16244 | 6896 | 1539 | 11537 | 19076 | -97.4% | 22.2 | **P+ better** |
| N vs P+ | write_p50_us | 663.6 | 20.6 | 778.8 | 57.9 | 43.4 | 208.0 | +17.4% | 2.65 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1717 | 739.3 | 2700 | 183.8 | 538.7 | 686.6 | +57.2% | 1.82 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1365 | 88.7 | 997.9 | 93.2 | 91.0 | 247.4 | -26.9% | 4.03 | **N better** |
| N vs P+ | pss_mib | 128.7 | 1.21 | 45.4 | 0.04 | 0.86 | 0.13 | -64.7% | 97.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 39893 | 4023 | -1816 | 474.9 | 2865 | 75161 | -104.6% | 14.6 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 173.4 | 7.35 | 129.7 | 2.63 | 5.52 | 15.8 | -25.2% | 7.92 | **P better** |
| N vs P | q1_p99_us | 414.6 | 29.9 | 445.0 | 51.8 | 42.3 | 112.7 | +7.4% | 0.72 | BELOW FLOOR |
| N vs P | q2_p50_us | 73700 | 5324 | 212.7 | 8.29 | 3765 | 4711 | -99.7% | 19.5 | **P better** |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 10539 | 2531 | 167.6 | 7.47 | 1789 | 1181 | -98.4% | 5.80 | **P better** |
| N vs P | q6_p50_us | 50023 | 1170 | 3397 | 84.9 | 829.3 | 7775 | -93.2% | 56.2 | **P better** |
| N vs P | c2_q1_p50_us | 129.9 | 17.9 | 56.9 | 6.38 | 13.5 | 13.3 | -56.2% | 5.43 | **P better** |
| N vs P | c2_q1_p99_us | 480.0 | 152.2 | 1161 | 243.2 | 202.8 | 318.1 | +142.0% | 3.36 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 82081 | 9149 | 155.6 | 31.2 | 6469 | 1125 | -99.8% | 12.7 | **P better** |
| N vs P | c4_q1_p50_us | 144.6 | 9.48 | 202.6 | 56.6 | 40.6 | 39.0 | +40.1% | 1.43 | no difference |
| N vs P | c4_q1_p99_us | 8022 | 493.8 | 2695 | 457.6 | 476.0 | 508.3 | -66.4% | 11.2 | **P better** |
| N vs P | c4_q2_p50_us | 121680 | 10435 | 333.3 | 34.1 | 7379 | 1489 | -99.7% | 16.4 | **P better** |
| N vs P | c4_q2_p99_us | 262616 | 16244 | 3719 | 513.9 | 11492 | 19125 | -98.6% | 22.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 663.6 | 20.6 | 171930 | 2648 | 1872 | 2706 | +25810.1% | 91.5 | **N better** |
| N vs P | write_p99_us | 1717 | 739.3 | 203621 | 17256 | 12213 | 13524 | +11759.3% | 16.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1365 | 88.7 | 5.67 | 0.14 | 62.7 | 140.5 | -99.6% | 21.7 | **N better** |
| N vs P | pss_mib | 128.7 | 1.21 | 45.2 | 0.04 | 0.86 | 0.12 | -64.8% | 97.4 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 39893 | 4023 | -1219 | 264.7 | 2851 | 75056 | -103.1% | 14.4 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 579.3 | 35.2 | 129.7 | 2.63 | 24.9 | 75.9 | -77.6% | 18.0 | **P better** |
| P+ vs P | q1_p99_us | 1934 | 530.6 | 445.0 | 51.8 | 377.0 | 2961 | -77.0% | 3.95 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 458.4 | 29.1 | 212.7 | 8.29 | 21.4 | 145.5 | -53.6% | 11.5 | **P better** |
| P+ vs P | q3_p50_us | 247.4 | 10.5 | 264.0 | 26.2 | 20.0 | 67.8 | +6.7% | 0.83 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 13000 | 223.1 | 1048 | 37.9 | 160.0 | 158.8 | -91.9% | 74.7 | **P better** |
| P+ vs P | q5_p50_us | 17437 | 2457 | 167.6 | 7.47 | 1737 | 4557 | -99.0% | 9.94 | **P better** |
| P+ vs P | q6_p50_us | 46092 | 2154 | 3397 | 84.9 | 1524 | 452.3 | -92.6% | 28.0 | **P better** |
| P+ vs P | c2_q1_p50_us | 621.8 | 62.6 | 56.9 | 6.38 | 44.5 | 145.5 | -90.9% | 12.7 | **P better** |
| P+ vs P | c2_q1_p99_us | 5225 | 289.3 | 1161 | 243.2 | 267.2 | 2508 | -77.8% | 15.2 | **P better** |
| P+ vs P | c2_q2_p50_us | 525.5 | 49.2 | 155.6 | 31.2 | 41.2 | 50.5 | -70.4% | 8.99 | **P better** |
| P+ vs P | c4_q1_p50_us | 1779 | 96.9 | 202.6 | 56.6 | 79.3 | 87.9 | -88.6% | 19.9 | **P better** |
| P+ vs P | c4_q1_p99_us | 7845 | 1314 | 2695 | 457.6 | 983.9 | 2059 | -65.7% | 5.23 | **P better** |
| P+ vs P | c4_q2_p50_us | 1545 | 157.9 | 333.3 | 34.1 | 114.2 | 300.9 | -78.4% | 10.6 | **P better** |
| P+ vs P | c4_q2_p99_us | 6896 | 1539 | 3719 | 513.9 | 1147 | 1366 | -46.1% | 2.77 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 778.8 | 57.9 | 171930 | 2648 | 1873 | 2713 | +21975.3% | 91.4 | **P+ better** |
| P+ vs P | write_p99_us | 2700 | 183.8 | 203621 | 17256 | 12203 | 13523 | +7441.9% | 16.5 | **P+ better** |
| P+ vs P | commits_per_s | 997.9 | 93.2 | 5.67 | 0.14 | 65.9 | 203.7 | -99.4% | 15.1 | **P+ better** |
| P+ vs P | pss_mib | 45.4 | 0.04 | 45.2 | 0.04 | 0.04 | 0.07 | -0.3% | 3.64 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -1816 | 474.9 | -1219 | 264.7 | 384.4 | 5552 | -32.9% | 1.55 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 129.7 | 2.63 | 129.8 | 1.60 | 2.18 | 15.7 | +0.1% | 0.04 | BELOW FLOOR |
| P vs M | q1_p99_us | 445.0 | 51.8 | 495.5 | 53.0 | 52.4 | 29.0 | +11.3% | 0.96 | no difference |
| P vs M | q2_p50_us | 212.7 | 8.29 | 207.1 | 12.1 | 10.4 | 60.1 | -2.7% | 0.54 | BELOW FLOOR |
| P vs M | q3_p50_us | 264.0 | 26.2 | 241.3 | 30.2 | 28.3 | 50.4 | -8.6% | 0.80 | BELOW FLOOR |
| P vs M | q4_p50_us | 1048 | 37.9 | 1006 | 15.6 | 29.0 | 127.1 | -4.0% | 1.44 | BELOW FLOOR |
| P vs M | q5_p50_us | 167.6 | 7.47 | 160.3 | 6.78 | 7.13 | 36.6 | -4.3% | 1.02 | BELOW FLOOR |
| P vs M | q6_p50_us | 3397 | 84.9 | 3468 | 129.6 | 109.6 | 199.2 | +2.1% | 0.65 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 56.9 | 6.38 | 63.8 | 11.5 | 9.32 | 32.6 | +12.2% | 0.75 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1161 | 243.2 | 1290 | 300.6 | 273.4 | 389.3 | +11.0% | 0.47 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 155.6 | 31.2 | 177.0 | 57.6 | 46.3 | 35.0 | +13.8% | 0.46 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 202.6 | 56.6 | 147.6 | 29.0 | 45.0 | 87.5 | -27.2% | 1.22 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2695 | 457.6 | 2281 | 408.0 | 433.5 | 476.7 | -15.3% | 0.95 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 333.3 | 34.1 | 283.2 | 55.7 | 46.2 | 54.9 | -15.0% | 1.09 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3719 | 513.9 | 2854 | 344.4 | 437.4 | 2048 | -23.3% | 1.98 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 171930 | 2648 | 171018 | 1824 | 2273 | 3063 | -0.5% | 0.40 | BELOW FLOOR |
| P vs M | write_p99_us | 203621 | 17256 | 192832 | 3945 | 12517 | 13764 | -5.3% | 0.86 | BELOW FLOOR |
| P vs M | commits_per_s | 5.67 | 0.14 | 5.71 | 0.09 | 0.12 | 0.10 | +0.7% | 0.35 | BELOW FLOOR |
| P vs M | pss_mib | 45.2 | 0.04 | 45.7 | 0.04 | 0.04 | 0.03 | +1.0% | 12.8 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -1219 | 264.7 | -778.2 | 252.9 | 258.8 | 3273 | -36.1% | 1.70 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 180.8 | 13.2 | 559.5 | 33.2 | 25.3 | 2.70 | +209.5% | 15.0 | **N better** |
| N vs P+ | q1_p99_us | 557.5 | 59.8 | 2344 | 509.8 | 363.0 | 138.7 | +320.5% | 4.92 | **N better** |
| N vs P+ | q2_p50_us | 76733 | 9907 | 452.4 | 48.8 | 7005 | 21901 | -99.4% | 10.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 8335 | 522.8 | 18792 | 2053 | 1498 | 2608 | +125.4% | 6.98 | **N better** |
| N vs P+ | q6_p50_us | 50541 | 284.0 | 47390 | 1336 | 965.7 | 2327 | -6.2% | 3.26 | no difference |
| N vs P+ | c2_q1_p50_us | 152.0 | 10.6 | 596.6 | 56.0 | 40.3 | 82.0 | +292.5% | 11.0 | **N better** |
| N vs P+ | c2_q1_p99_us | 1852 | 794.7 | 5442 | 1379 | 1125 | 882.2 | +193.9% | 3.19 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 72362 | 3269 | 499.0 | 33.1 | 2312 | 1671 | -99.3% | 31.1 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 145.0 | 21.4 | 1590 | 46.2 | 36.0 | 143.7 | +996.5% | 40.1 | **N better** |
| N vs P+ | c4_q1_p99_us | 8661 | 755.2 | 7833 | 1335 | 1085 | 752.6 | -9.6% | 0.76 | no difference |
| N vs P+ | c4_q2_p50_us | 121256 | 8822 | 1456 | 65.1 | 6238 | 5429 | -98.8% | 19.2 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 263854 | 19262 | 7199 | 2879 | 13772 | 15314 | -97.3% | 18.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 687.7 | 23.0 | 824.2 | 26.9 | 25.0 | 164.9 | +19.8% | 5.46 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1268 | 146.9 | 2741 | 63.8 | 113.3 | 1490 | +116.1% | 13.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 1336 | 65.6 | 999.6 | 28.4 | 50.6 | 435.7 | -25.2% | 6.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | pss_mib | 128.7 | 0.07 | 45.5 | 0.02 | 0.05 | 0.06 | -64.7% | 1585 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 33886 | 4757 | -798.9 | 168.7 | 3366 | 35773 | -102.4% | 10.3 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 180.8 | 13.2 | 134.9 | 5.48 | 10.1 | 3.32 | -25.4% | 4.54 | **P better** |
| N vs P | q1_p99_us | 557.5 | 59.8 | 469.1 | 64.6 | 62.2 | 153.1 | -15.8% | 1.42 | BELOW FLOOR |
| N vs P | q2_p50_us | 76733 | 9907 | 214.3 | 8.96 | 7005 | 21900 | -99.7% | 10.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 8335 | 522.8 | 167.2 | 6.00 | 369.7 | 292.0 | -98.0% | 22.1 | **P better** |
| N vs P | q6_p50_us | 50541 | 284.0 | 3541 | 98.0 | 212.4 | 255.7 | -93.0% | 221.2 | **P better** |
| N vs P | c2_q1_p50_us | 152.0 | 10.6 | 116.5 | 20.9 | 16.5 | 7.77 | -23.3% | 2.14 | no difference |
| N vs P | c2_q1_p99_us | 1852 | 794.7 | 1941 | 498.6 | 663.3 | 850.6 | +4.8% | 0.14 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 72362 | 3269 | 207.7 | 48.5 | 2312 | 1670 | -99.7% | 31.2 | **P better** |
| N vs P | c4_q1_p50_us | 145.0 | 21.4 | 169.2 | 16.1 | 18.9 | 90.4 | +16.7% | 1.28 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 8661 | 755.2 | 2452 | 211.9 | 554.6 | 594.2 | -71.7% | 11.2 | **P better** |
| N vs P | c4_q2_p50_us | 121256 | 8822 | 311.3 | 50.9 | 6238 | 5430 | -99.7% | 19.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 263854 | 19262 | 3400 | 1064 | 13641 | 15343 | -98.7% | 19.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 687.7 | 23.0 | 172042 | 1696 | 1199 | 1558 | +24916.4% | 142.9 | **N better** |
| N vs P | write_p99_us | 1268 | 146.9 | 203981 | 9981 | 7059 | 696.1 | +15984.6% | 28.7 | **N better** |
| N vs P | commits_per_s | 1336 | 65.6 | 5.71 | 0.12 | 46.4 | 153.3 | -99.6% | 28.7 | **N better** |
| N vs P | pss_mib | 128.7 | 0.07 | 45.2 | 0.04 | 0.06 | 0.06 | -64.8% | 1424 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 33886 | 4757 | -840.7 | 180.7 | 3366 | 35772 | -102.5% | 10.3 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 559.5 | 33.2 | 134.9 | 5.48 | 23.8 | 2.35 | -75.9% | 17.8 | **P better** |
| P+ vs P | q1_p99_us | 2344 | 509.8 | 469.1 | 64.6 | 363.4 | 86.1 | -80.0% | 5.16 | **P better** |
| P+ vs P | q2_p50_us | 452.4 | 48.8 | 214.3 | 8.96 | 35.0 | 53.3 | -52.6% | 6.80 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 277.4 | 23.2 | 265.2 | 35.8 | 30.2 | 87.6 | -4.4% | 0.40 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 12804 | 460.4 | 1036 | 52.1 | 327.6 | 264.3 | -91.9% | 35.9 | **P better** |
| P+ vs P | q5_p50_us | 18792 | 2053 | 167.2 | 6.00 | 1452 | 2592 | -99.1% | 12.8 | **P better** |
| P+ vs P | q6_p50_us | 47390 | 1336 | 3541 | 98.0 | 947.1 | 2331 | -92.5% | 46.3 | **P better** |
| P+ vs P | c2_q1_p50_us | 596.6 | 56.0 | 116.5 | 20.9 | 42.3 | 82.1 | -80.5% | 11.4 | **P better** |
| P+ vs P | c2_q1_p99_us | 5442 | 1379 | 1941 | 498.6 | 1037 | 251.0 | -64.3% | 3.38 | **P better** |
| P+ vs P | c2_q2_p50_us | 499.0 | 33.1 | 207.7 | 48.5 | 41.5 | 74.2 | -58.4% | 7.02 | **P better** |
| P+ vs P | c4_q1_p50_us | 1590 | 46.2 | 169.2 | 16.1 | 34.6 | 160.2 | -89.4% | 41.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 7833 | 1335 | 2452 | 211.9 | 956.0 | 929.5 | -68.7% | 5.63 | **P better** |
| P+ vs P | c4_q2_p50_us | 1456 | 65.1 | 311.3 | 50.9 | 58.4 | 173.3 | -78.6% | 19.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 7199 | 2879 | 3400 | 1064 | 2170 | 8671 | -52.8% | 1.75 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 824.2 | 26.9 | 172042 | 1696 | 1199 | 1567 | +20775.1% | 142.8 | **P+ better** |
| P+ vs P | write_p99_us | 2741 | 63.8 | 203981 | 9981 | 7058 | 1642 | +7343.1% | 28.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 999.6 | 28.4 | 5.71 | 0.12 | 20.1 | 407.8 | -99.4% | 49.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | pss_mib | 45.5 | 0.02 | 45.2 | 0.04 | 0.03 | 0.02 | -0.5% | 6.74 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -798.9 | 168.7 | -840.7 | 180.7 | 174.8 | 2550 | +5.2% | 0.24 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 134.9 | 5.48 | 130.3 | 5.52 | 5.50 | 10.5 | -3.4% | 0.84 | BELOW FLOOR |
| P vs M | q1_p99_us | 469.1 | 64.6 | 466.2 | 39.0 | 53.4 | 76.2 | -0.6% | 0.05 | BELOW FLOOR |
| P vs M | q2_p50_us | 214.3 | 8.96 | 216.1 | 6.60 | 7.87 | 26.0 | +0.9% | 0.23 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 265.2 | 35.8 | 251.8 | 39.3 | 37.6 | 109.8 | -5.0% | 0.36 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q4_p50_us | 1036 | 52.1 | 1015 | 20.4 | 39.5 | 178.6 | -2.1% | 0.54 | BELOW FLOOR |
| P vs M | q5_p50_us | 167.2 | 6.00 | 173.1 | 9.86 | 8.16 | 29.1 | +3.5% | 0.72 | BELOW FLOOR |
| P vs M | q6_p50_us | 3541 | 98.0 | 3512 | 113.7 | 106.1 | 277.4 | -0.8% | 0.27 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 116.5 | 20.9 | 54.0 | 3.09 | 14.9 | 34.6 | -53.6% | 4.19 | **M better** |
| P vs M | c2_q1_p99_us | 1941 | 498.6 | 1084 | 105.5 | 360.3 | 338.5 | -44.2% | 2.38 | no difference |
| P vs M | c2_q2_p50_us | 207.7 | 48.5 | 121.3 | 12.6 | 35.4 | 52.9 | -41.6% | 2.44 | no difference |
| P vs M | c4_q1_p50_us | 169.2 | 16.1 | 149.1 | 35.3 | 27.4 | 81.6 | -11.9% | 0.73 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2452 | 211.9 | 2293 | 360.3 | 295.5 | 890.0 | -6.5% | 0.54 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 311.3 | 50.9 | 275.4 | 38.9 | 45.3 | 150.2 | -11.5% | 0.79 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 3400 | 1064 | 3028 | 1034 | 1049 | 6176 | -11.0% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 172042 | 1696 | 172682 | 1622 | 1659 | 1627 | +0.4% | 0.39 | BELOW FLOOR |
| P vs M | write_p99_us | 203981 | 9981 | 205291 | 19498 | 15489 | 37290 | +0.6% | 0.08 | BELOW FLOOR |
| P vs M | commits_per_s | 5.71 | 0.12 | 5.70 | 0.11 | 0.11 | 0.14 | -0.1% | 0.08 | BELOW FLOOR |
| P vs M | pss_mib | 45.2 | 0.04 | 45.4 | 0.13 | 0.10 | 0.03 | +0.4% | 1.72 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -840.7 | 180.7 | -632.1 | 129.9 | 157.4 | 2244 | -24.8% | 1.33 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 169.4 | 9.43 | 520.1 | 19.4 | 15.2 | 29.1 | +207.0% | 23.0 | **N better** |
| N vs P+ | q1_p99_us | 446.5 | 37.7 | 1335 | 88.2 | 67.9 | 720.2 | +198.9% | 13.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 71768 | 11748 | 412.9 | 15.7 | 8307 | 1239 | -99.4% | 8.59 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 17855 | 9946 | 31004 | 8626 | 9309 | 1197 | +73.6% | 1.41 | no difference |
| N vs P+ | q6_p50_us | 49198 | 540.3 | 44919 | 934.8 | 763.4 | 650.1 | -8.7% | 5.61 | no difference |
| N vs P+ | c2_q1_p50_us | 125.2 | 17.3 | 565.3 | 31.9 | 25.7 | 46.9 | +351.3% | 17.2 | **N better** |
| N vs P+ | c2_q1_p99_us | 432.8 | 72.3 | 5010 | 857.7 | 608.7 | 6398 | +1057.7% | 7.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 71649 | 5050 | 496.7 | 22.8 | 3571 | 32631 | -99.3% | 19.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c4_q1_p50_us | 117.7 | 15.4 | 1593 | 81.0 | 58.3 | 185.8 | +1253.7% | 25.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 7888 | 719.6 | 7216 | 948.4 | 841.8 | 1836 | -8.5% | 0.80 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 113506 | 8359 | 1443 | 71.7 | 5911 | 3438 | -98.7% | 19.0 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 257740 | 6639 | 6798 | 501.3 | 4708 | 8604 | -97.4% | 53.3 | **P+ better** |
| N vs P+ | write_p50_us | 794.2 | 16.8 | 741.5 | 26.8 | 22.4 | 115.2 | -6.6% | 2.35 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1281 | 186.6 | 3037 | 297.1 | 248.1 | 448.0 | +137.2% | 7.08 | **N better** |
| N vs P+ | commits_per_s | 1223 | 36.6 | 1082 | 93.4 | 70.9 | 99.4 | -11.5% | 1.99 | no difference |
| N vs P+ | pss_mib | 202.5 | 53.2 | 45.4 | 0.04 | 37.6 | 1.07 | -77.6% | 4.17 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 63334 | 5358 | -791.2 | 124.9 | 3790 | 67601 | -101.2% | 16.9 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 169.4 | 9.43 | 127.6 | 4.75 | 7.46 | 3.49 | -24.7% | 5.60 | **P better** |
| N vs P | q1_p99_us | 446.5 | 37.7 | 401.1 | 16.3 | 29.1 | 72.0 | -10.2% | 1.56 | BELOW FLOOR |
| N vs P | q2_p50_us | 71768 | 11748 | 213.5 | 13.5 | 8307 | 1238 | -99.7% | 8.61 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 17855 | 9946 | 185.3 | 15.8 | 7033 | 377.1 | -99.0% | 2.51 | no difference |
| N vs P | q6_p50_us | 49198 | 540.3 | 3443 | 53.4 | 383.9 | 639.4 | -93.0% | 119.2 | **P better** |
| N vs P | c2_q1_p50_us | 125.2 | 17.3 | 71.3 | 19.2 | 18.3 | 43.9 | -43.1% | 2.95 | no difference |
| N vs P | c2_q1_p99_us | 432.8 | 72.3 | 1136 | 341.8 | 247.1 | 6562 | +162.5% | 2.85 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 71649 | 5050 | 166.1 | 21.1 | 3571 | 32631 | -99.8% | 20.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c4_q1_p50_us | 117.7 | 15.4 | 139.9 | 13.9 | 14.7 | 86.5 | +18.8% | 1.51 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 7888 | 719.6 | 2388 | 435.3 | 594.7 | 1643 | -69.7% | 9.25 | **P better** |
| N vs P | c4_q2_p50_us | 113506 | 8359 | 267.6 | 20.4 | 5911 | 3440 | -99.8% | 19.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 257740 | 6639 | 3355 | 821.9 | 4730 | 8336 | -98.7% | 53.8 | **P better** |
| N vs P | write_p50_us | 794.2 | 16.8 | 168624 | 3102 | 2193 | 2782 | +21133.1% | 76.5 | **N better** |
| N vs P | write_p99_us | 1281 | 186.6 | 199312 | 16710 | 11817 | 3688 | +15463.8% | 16.8 | **N better** |
| N vs P | commits_per_s | 1223 | 36.6 | 5.81 | 0.12 | 25.9 | 0.32 | -99.5% | 47.0 | **N better** |
| N vs P | pss_mib | 202.5 | 53.2 | 45.2 | 0.04 | 37.6 | 1.07 | -77.7% | 4.18 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 63334 | 5358 | -785.1 | 217.3 | 3792 | 67604 | -101.2% | 16.9 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 520.1 | 19.4 | 127.6 | 4.75 | 14.1 | 29.2 | -75.5% | 27.8 | **P better** |
| P+ vs P | q1_p99_us | 1335 | 88.2 | 401.1 | 16.3 | 63.5 | 723.7 | -69.9% | 14.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 412.9 | 15.7 | 213.5 | 13.5 | 14.6 | 61.2 | -48.3% | 13.7 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 254.1 | 23.3 | 248.5 | 41.8 | 33.8 | 12.1 | -2.2% | 0.17 | BELOW FLOOR |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q5_p50_us | 31004 | 8626 | 185.3 | 15.8 | 6100 | 1136 | -99.4% | 5.05 | **P better** |
| P+ vs P | q6_p50_us | 44919 | 934.8 | 3443 | 53.4 | 662.1 | 137.1 | -92.3% | 62.6 | **P better** |
| P+ vs P | c2_q1_p50_us | 565.3 | 31.9 | 71.3 | 19.2 | 26.3 | 52.5 | -87.4% | 18.8 | **P better** |
| P+ vs P | c2_q1_p99_us | 5010 | 857.7 | 1136 | 341.8 | 652.9 | 1458 | -77.3% | 5.93 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 496.7 | 22.8 | 166.1 | 21.1 | 22.0 | 84.3 | -66.6% | 15.0 | **P better** |
| P+ vs P | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c4_q1_p50_us | 1593 | 81.0 | 139.9 | 13.9 | 58.1 | 205.0 | -91.2% | 25.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 7216 | 948.4 | 2388 | 435.3 | 737.9 | 854.1 | -66.9% | 6.54 | **P better** |
| P+ vs P | c4_q2_p50_us | 1443 | 71.7 | 267.6 | 20.4 | 52.7 | 197.0 | -81.5% | 22.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 6798 | 501.3 | 3355 | 821.9 | 680.8 | 2191 | -50.7% | 5.06 | **P better** |
| P+ vs P | write_p50_us | 741.5 | 26.8 | 168624 | 3102 | 2194 | 2782 | +22642.0% | 76.5 | **P+ better** |
| P+ vs P | write_p99_us | 3037 | 297.1 | 199312 | 16710 | 11818 | 3666 | +6462.0% | 16.6 | **P+ better** |
| P+ vs P | commits_per_s | 1082 | 93.4 | 5.81 | 0.12 | 66.0 | 99.4 | -99.5% | 16.3 | **P+ better** |
| P+ vs P | pss_mib | 45.4 | 0.04 | 45.2 | 0.04 | 0.04 | 0.03 | -0.4% | 4.31 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -791.2 | 124.9 | -785.1 | 217.3 | 177.2 | 2630 | -0.8% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 127.6 | 4.75 | 124.1 | 4.57 | 4.66 | 9.15 | -2.8% | 0.75 | BELOW FLOOR |
| P vs M | q1_p99_us | 401.1 | 16.3 | 420.4 | 25.4 | 21.3 | 72.0 | +4.8% | 0.91 | BELOW FLOOR |
| P vs M | q2_p50_us | 213.5 | 13.5 | 208.8 | 14.6 | 14.0 | 33.1 | -2.2% | 0.33 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 248.5 | 41.8 | 243.2 | 12.3 | 30.8 | 13.4 | -2.1% | 0.17 | BELOW FLOOR |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q5_p50_us | 185.3 | 15.8 | 180.9 | 12.4 | 14.2 | 14.5 | -2.4% | 0.31 | BELOW FLOOR |
| P vs M | q6_p50_us | 3443 | 53.4 | 3420 | 52.9 | 53.2 | 62.2 | -0.7% | 0.43 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 71.3 | 19.2 | 117.5 | 12.5 | 16.2 | 35.3 | +64.8% | 2.85 | no difference |
| P vs M | c2_q1_p99_us | 1136 | 341.8 | 1297 | 391.0 | 367.2 | 1462 | +14.1% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 166.1 | 21.1 | 188.6 | 12.9 | 17.5 | 47.4 | +13.6% | 1.29 | BELOW FLOOR |
| P vs M | c2_q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c4_q1_p50_us | 139.9 | 13.9 | 187.8 | 22.8 | 18.9 | 111.8 | +34.3% | 2.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2388 | 435.3 | 2522 | 514.0 | 476.3 | 429.8 | +5.6% | 0.28 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 267.6 | 20.4 | 303.0 | 26.1 | 23.4 | 155.3 | +13.2% | 1.51 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 3355 | 821.9 | 3661 | 642.1 | 737.5 | 988.6 | +9.1% | 0.42 | BELOW FLOOR |
| P vs M | write_p50_us | 168624 | 3102 | 170119 | 946.4 | 2293 | 3006 | +0.9% | 0.65 | BELOW FLOOR |
| P vs M | write_p99_us | 199312 | 16710 | 204207 | 8160 | 13150 | 3674 | +2.5% | 0.37 | no difference |
| P vs M | commits_per_s | 5.81 | 0.12 | 5.76 | 0.07 | 0.10 | 0.11 | -0.8% | 0.50 | BELOW FLOOR |
| P vs M | pss_mib | 45.2 | 0.04 | 45.5 | 0.03 | 0.04 | 0.42 | +0.5% | 6.14 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -785.1 | 217.3 | -734.8 | 133.3 | 180.3 | 5772 | -6.4% | 0.28 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 185.7 | 7.97 | 567.1 | 14.2 | 11.5 | 34.0 | +205.4% | 33.1 | **N better** |
| N vs P+ | q1_p99_us | 536.7 | 117.1 | 1986 | 612.6 | 441.0 | 2604 | +270.1% | 3.29 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 70906 | 7121 | 456.1 | 9.34 | 5035 | 7050 | -99.4% | 14.0 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 12773 | 4713 | 18670 | 3599 | 4193 | 3524 | +46.2% | 1.41 | no difference |
| N vs P+ | q6_p50_us | 51357 | 1098 | 49445 | 1360 | 1236 | 4547 | -3.7% | 1.55 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 146.8 | 3.68 | 590.9 | 32.1 | 22.8 | 63.5 | +302.5% | 19.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 535.9 | 152.2 | 5152 | 501.3 | 370.5 | 715.8 | +861.5% | 12.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 74967 | 4412 | 514.2 | 18.1 | 3120 | 3357 | -99.3% | 23.9 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 150.3 | 16.7 | 1653 | 93.1 | 66.9 | 414.2 | +999.9% | 22.5 | **N better** |
| N vs P+ | c4_q1_p99_us | 7532 | 972.9 | 7929 | 1632 | 1343 | 2587 | +5.3% | 0.30 | BELOW FLOOR |
| N vs P+ | c4_q2_p50_us | 130276 | 13747 | 1562 | 78.2 | 9720 | 14590 | -98.8% | 13.2 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 268784 | 8154 | 7747 | 3518 | 6280 | 95220 | -97.1% | 41.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 688.7 | 16.2 | 806.9 | 16.2 | 16.2 | 145.2 | +17.2% | 7.28 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1133 | 106.0 | 2893 | 257.7 | 197.1 | 950.5 | +155.3% | 8.93 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1348 | 35.1 | 996.7 | 53.7 | 45.4 | 124.0 | -26.0% | 7.73 | **N better** |
| N vs P+ | pss_mib | 222.5 | 0.45 | 45.5 | 0.03 | 0.32 | 8.37 | -79.6% | 553.9 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 62683 | 5526 | -752.6 | 192.5 | 3910 | 16082 | -101.2% | 16.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 185.7 | 7.97 | 130.3 | 3.58 | 6.18 | 34.0 | -29.8% | 8.97 | **P better** |
| N vs P | q1_p99_us | 536.7 | 117.1 | 500.0 | 112.7 | 114.9 | 172.1 | -6.8% | 0.32 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 70906 | 7121 | 210.4 | 4.78 | 5035 | 7050 | -99.7% | 14.0 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 12773 | 4713 | 176.5 | 8.03 | 3332 | 59.7 | -98.6% | 3.78 | **P better** |
| N vs P | q6_p50_us | 51357 | 1098 | 3646 | 177.9 | 786.8 | 1459 | -92.9% | 60.6 | **P better** |
| N vs P | c2_q1_p50_us | 146.8 | 3.68 | 115.1 | 32.2 | 22.9 | 63.5 | -21.6% | 1.38 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 535.9 | 152.2 | 1286 | 262.8 | 214.8 | 601.5 | +140.1% | 3.49 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 74967 | 4412 | 228.4 | 69.5 | 3120 | 3357 | -99.7% | 24.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 150.3 | 16.7 | 165.1 | 23.7 | 20.5 | 35.9 | +9.8% | 0.72 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 7532 | 972.9 | 2575 | 277.2 | 715.3 | 1386 | -65.8% | 6.93 | **P better** |
| N vs P | c4_q2_p50_us | 130276 | 13747 | 315.8 | 38.7 | 9720 | 14587 | -99.8% | 13.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 268784 | 8154 | 3270 | 671.0 | 5786 | 95225 | -98.8% | 45.9 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 688.7 | 16.2 | 171101 | 1252 | 885.4 | 606.2 | +24743.5% | 192.5 | **N better** |
| N vs P | write_p99_us | 1133 | 106.0 | 212711 | 10768 | 7615 | 3898 | +18674.8% | 27.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1348 | 35.1 | 5.69 | 0.05 | 24.8 | 97.5 | -99.6% | 54.0 | **N better** |
| N vs P | pss_mib | 222.5 | 0.45 | 45.4 | 0.03 | 0.32 | 8.37 | -79.6% | 553.7 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 62683 | 5526 | -810.4 | 160.9 | 3909 | 16105 | -101.3% | 16.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 567.1 | 14.2 | 130.3 | 3.58 | 10.4 | 11.6 | -77.0% | 42.1 | **P better** |
| P+ vs P | q1_p99_us | 1986 | 612.6 | 500.0 | 112.7 | 440.4 | 2600 | -74.8% | 3.37 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 456.1 | 9.34 | 210.4 | 4.78 | 7.42 | 30.0 | -53.9% | 33.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 278.2 | 23.3 | 238.9 | 20.5 | 21.9 | 34.3 | -14.1% | 1.79 | no difference |
| P+ vs P | q4_p50_us | 12811 | 367.4 | 1022 | 32.2 | 260.8 | 161.7 | -92.0% | 45.2 | **P better** |
| P+ vs P | q5_p50_us | 18670 | 3599 | 176.5 | 8.03 | 2545 | 3524 | -99.1% | 7.27 | **P better** |
| P+ vs P | q6_p50_us | 49445 | 1360 | 3646 | 177.9 | 970.0 | 4320 | -92.6% | 47.2 | **P better** |
| P+ vs P | c2_q1_p50_us | 590.9 | 32.1 | 115.1 | 32.2 | 32.1 | 9.34 | -80.5% | 14.8 | **P better** |
| P+ vs P | c2_q1_p99_us | 5152 | 501.3 | 1286 | 262.8 | 400.3 | 454.0 | -75.0% | 9.66 | **P better** |
| P+ vs P | c2_q2_p50_us | 514.2 | 18.1 | 228.4 | 69.5 | 50.8 | 72.1 | -55.6% | 5.63 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1653 | 93.1 | 165.1 | 23.7 | 68.0 | 415.3 | -90.0% | 21.9 | **P better** |
| P+ vs P | c4_q1_p99_us | 7929 | 1632 | 2575 | 277.2 | 1170 | 2305 | -67.5% | 4.57 | **P better** |
| P+ vs P | c4_q2_p50_us | 1562 | 78.2 | 315.8 | 38.7 | 61.7 | 383.4 | -79.8% | 20.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 7747 | 3518 | 3270 | 671.0 | 2533 | 1035 | -57.8% | 1.77 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 806.9 | 16.2 | 171101 | 1252 | 885.4 | 616.7 | +21103.8% | 192.3 | **P+ better** |
| P+ vs P | write_p99_us | 2893 | 257.7 | 212711 | 10768 | 7616 | 3872 | +7253.4% | 27.5 | **P+ better** |
| P+ vs P | commits_per_s | 996.7 | 53.7 | 5.69 | 0.05 | 38.0 | 76.7 | -99.4% | 26.1 | **P+ better** |
| P+ vs P | pss_mib | 45.5 | 0.03 | 45.4 | 0.03 | 0.03 | 0.08 | -0.1% | 1.85 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -752.6 | 192.5 | -810.4 | 160.9 | 177.4 | 3007 | +7.7% | 0.33 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 130.3 | 3.58 | 132.6 | 3.30 | 3.44 | 9.21 | +1.7% | 0.66 | BELOW FLOOR |
| P vs M | q1_p99_us | 500.0 | 112.7 | 648.6 | 234.9 | 184.2 | 214.2 | +29.7% | 0.81 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 210.4 | 4.78 | 214.1 | 5.21 | 5.00 | 24.3 | +1.8% | 0.74 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 238.9 | 20.5 | 251.0 | 25.3 | 23.0 | 39.1 | +5.1% | 0.53 | BELOW FLOOR |
| P vs M | q4_p50_us | 1022 | 32.2 | 1048 | 25.5 | 29.0 | 88.1 | +2.6% | 0.90 | BELOW FLOOR |
| P vs M | q5_p50_us | 176.5 | 8.03 | 187.9 | 6.56 | 7.33 | 40.6 | +6.5% | 1.56 | BELOW FLOOR |
| P vs M | q6_p50_us | 3646 | 177.9 | 3483 | 82.7 | 138.8 | 244.3 | -4.5% | 1.18 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 115.1 | 32.2 | 54.2 | 2.70 | 22.9 | 9.77 | -52.9% | 2.66 | no difference |
| P vs M | c2_q1_p99_us | 1286 | 262.8 | 1062 | 107.7 | 200.9 | 1939 | -17.4% | 1.12 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 228.4 | 69.5 | 138.0 | 7.63 | 49.4 | 79.6 | -39.6% | 1.83 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 165.1 | 23.7 | 159.7 | 18.8 | 21.4 | 143.2 | -3.2% | 0.25 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2575 | 277.2 | 2751 | 541.4 | 430.1 | 598.7 | +6.8% | 0.41 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 315.8 | 38.7 | 293.5 | 26.4 | 33.1 | 243.9 | -7.1% | 0.67 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p99_us | 3270 | 671.0 | 4122 | 873.6 | 778.9 | 1808 | +26.1% | 1.09 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 171101 | 1252 | 172975 | 1756 | 1525 | 839.3 | +1.1% | 1.23 | no difference |
| P vs M | write_p99_us | 212711 | 10768 | 202476 | 12307 | 11563 | 10671 | -4.8% | 0.89 | BELOW FLOOR |
| P vs M | commits_per_s | 5.69 | 0.05 | 5.68 | 0.06 | 0.05 | 0.06 | -0.1% | 0.12 | BELOW FLOOR |
| P vs M | pss_mib | 45.4 | 0.03 | 45.6 | 0.04 | 0.04 | 0.10 | +0.4% | 4.53 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -810.4 | 160.9 | -704.9 | 144.0 | 152.7 | 3291 | -13.0% | 0.69 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 174.1 | 7.68 | 543.2 | 23.8 | 17.7 | 11.6 | +212.0% | 20.8 | **N better** |
| N vs P+ | q1_p99_us | 493.0 | 74.4 | 1457 | 172.2 | 132.6 | 503.0 | +195.6% | 7.27 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 67878 | 4562 | 427.2 | 15.5 | 3226 | 9039 | -99.4% | 20.9 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 27758 | 458.6 | 39586 | 1232 | 929.4 | 1219 | +42.6% | 12.7 | **N better** |
| N vs P+ | q6_p50_us | 49971 | 319.7 | 47070 | 1011 | 749.9 | 4760 | -5.8% | 3.87 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 122.0 | 28.8 | 600.7 | 48.7 | 40.0 | 34.7 | +392.5% | 12.0 | **N better** |
| N vs P+ | c2_q1_p99_us | 437.1 | 70.0 | 5229 | 332.5 | 240.3 | 577.8 | +1096.2% | 19.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 78515 | 8094 | 482.4 | 15.1 | 5723 | 14971 | -99.4% | 13.6 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 143.1 | 7.74 | 1618 | 76.9 | 54.7 | 209.0 | +1030.8% | 27.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 8341 | 458.6 | 7576 | 1149 | 874.9 | 2721 | -9.2% | 0.87 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 128765 | 14319 | 1479 | 98.7 | 10125 | 3509 | -98.9% | 12.6 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 262686 | 15689 | 8222 | 1200 | 11126 | 6243 | -96.9% | 22.9 | **P+ better** |
| N vs P+ | write_p50_us | 675.3 | 27.5 | 736.6 | 60.7 | 47.1 | 63.0 | +9.1% | 1.30 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1038 | 139.3 | 3180 | 557.8 | 406.5 | 1596 | +206.4% | 5.27 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1429 | 96.6 | 1034 | 93.7 | 95.2 | 27.5 | -27.7% | 4.15 | **N better** |
| N vs P+ | pss_mib | 128.7 | 0.10 | 45.2 | 0.03 | 0.07 | 0.21 | -64.9% | 1124 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 33839 | 6901 | -790.7 | 148.7 | 4881 | 10427 | -102.3% | 7.09 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 174.1 | 7.68 | 134.6 | 4.95 | 6.46 | 4.10 | -22.7% | 6.11 | **P better** |
| N vs P | q1_p99_us | 493.0 | 74.4 | 458.4 | 55.3 | 65.6 | 466.9 | -7.0% | 0.53 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 67878 | 4562 | 223.7 | 10.7 | 3226 | 9039 | -99.7% | 21.0 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 27758 | 458.6 | 175.3 | 14.5 | 324.4 | 1184 | -99.4% | 85.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 49971 | 319.7 | 3589 | 152.5 | 250.5 | 3164 | -92.8% | 185.2 | **P better** |
| N vs P | c2_q1_p50_us | 122.0 | 28.8 | 102.3 | 47.6 | 39.3 | 35.9 | -16.2% | 0.50 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 437.1 | 70.0 | 1597 | 374.6 | 269.4 | 572.2 | +265.4% | 4.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 78515 | 8094 | 229.0 | 46.6 | 5723 | 14971 | -99.7% | 13.7 | **P better** |
| N vs P | c4_q1_p50_us | 143.1 | 7.74 | 169.8 | 44.5 | 31.9 | 43.9 | +18.7% | 0.84 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 8341 | 458.6 | 2734 | 205.2 | 355.2 | 542.2 | -67.2% | 15.8 | **P better** |
| N vs P | c4_q2_p50_us | 128765 | 14319 | 323.4 | 99.7 | 10125 | 3503 | -99.7% | 12.7 | **P better** |
| N vs P | c4_q2_p99_us | 262686 | 15689 | 4165 | 917.2 | 11113 | 6236 | -98.4% | 23.3 | **P better** |
| N vs P | write_p50_us | 675.3 | 27.5 | 176800 | 3105 | 2196 | 124.2 | +26080.2% | 80.2 | **N better** |
| N vs P | write_p99_us | 1038 | 139.3 | 208524 | 16662 | 11782 | 2012 | +19990.8% | 17.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1429 | 96.6 | 5.51 | 0.11 | 68.3 | 3.36 | -99.6% | 20.8 | **N better** |
| N vs P | pss_mib | 128.7 | 0.10 | 45.3 | 0.05 | 0.08 | 5.52 | -64.8% | 1074 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 33839 | 6901 | -785.4 | 187.3 | 4882 | 11205 | -102.3% | 7.09 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 543.2 | 23.8 | 134.6 | 4.95 | 17.2 | 12.3 | -75.2% | 23.7 | **P better** |
| P+ vs P | q1_p99_us | 1457 | 172.2 | 458.4 | 55.3 | 127.9 | 388.4 | -68.5% | 7.81 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 427.2 | 15.5 | 223.7 | 10.7 | 13.3 | 40.5 | -47.6% | 15.3 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 271.9 | 19.8 | 241.0 | 25.3 | 22.7 | 82.3 | -11.4% | 1.36 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 12758 | 221.5 | 1026 | 40.1 | 159.2 | 39.4 | -92.0% | 73.7 | **P better** |
| P+ vs P | q5_p50_us | 39586 | 1232 | 175.3 | 14.5 | 871.1 | 304.8 | -99.6% | 45.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 47070 | 1011 | 3589 | 152.5 | 723.1 | 3587 | -92.4% | 60.1 | **P better** |
| P+ vs P | c2_q1_p50_us | 600.7 | 48.7 | 102.3 | 47.6 | 48.2 | 45.6 | -83.0% | 10.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5229 | 332.5 | 1597 | 374.6 | 354.2 | 496.1 | -69.5% | 10.3 | **P better** |
| P+ vs P | c2_q2_p50_us | 482.4 | 15.1 | 229.0 | 46.6 | 34.7 | 45.9 | -52.5% | 7.31 | **P better** |
| P+ vs P | c4_q1_p50_us | 1618 | 76.9 | 169.8 | 44.5 | 62.8 | 205.2 | -89.5% | 23.1 | **P better** |
| P+ vs P | c4_q1_p99_us | 7576 | 1149 | 2734 | 205.2 | 825.5 | 2708 | -63.9% | 5.87 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1479 | 98.7 | 323.4 | 99.7 | 99.2 | 219.8 | -78.1% | 11.6 | **P better** |
| P+ vs P | c4_q2_p99_us | 8222 | 1200 | 4165 | 917.2 | 1068 | 519.9 | -49.3% | 3.80 | **P better** |
| P+ vs P | write_p50_us | 736.6 | 60.7 | 176800 | 3105 | 2196 | 111.1 | +23903.2% | 80.2 | **P+ better** |
| P+ vs P | write_p99_us | 3180 | 557.8 | 208524 | 16662 | 11788 | 1230 | +6457.4% | 17.4 | **P+ better** |
| P+ vs P | commits_per_s | 1034 | 93.7 | 5.51 | 0.11 | 66.2 | 27.3 | -99.5% | 15.5 | **P+ better** |
| P+ vs P | pss_mib | 45.2 | 0.03 | 45.3 | 0.05 | 0.04 | 5.51 | +0.3% | 2.85 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -790.7 | 148.7 | -785.4 | 187.3 | 169.1 | 4908 | -0.7% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 134.6 | 4.95 | 132.3 | 3.29 | 4.21 | 14.3 | -1.7% | 0.56 | BELOW FLOOR |
| P vs M | q1_p99_us | 458.4 | 55.3 | 471.0 | 97.6 | 79.3 | 241.6 | +2.7% | 0.16 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 223.7 | 10.7 | 210.5 | 8.82 | 9.83 | 42.2 | -5.9% | 1.34 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 241.0 | 25.3 | 239.5 | 24.0 | 24.6 | 64.8 | -0.6% | 0.06 | BELOW FLOOR |
| P vs M | q4_p50_us | 1026 | 40.1 | 1022 | 23.3 | 32.8 | 43.1 | -0.4% | 0.14 | BELOW FLOOR |
| P vs M | q5_p50_us | 175.3 | 14.5 | 179.3 | 16.7 | 15.7 | 76.4 | +2.2% | 0.25 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 3589 | 152.5 | 3521 | 115.9 | 135.4 | 413.3 | -1.9% | 0.50 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 102.3 | 47.6 | 54.6 | 1.71 | 33.7 | 32.9 | -46.6% | 1.42 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1597 | 374.6 | 1076 | 62.5 | 268.5 | 381.6 | -32.6% | 1.94 | no difference |
| P vs M | c2_q2_p50_us | 229.0 | 46.6 | 133.6 | 19.2 | 35.7 | 19.0 | -41.7% | 2.67 | no difference |
| P vs M | c4_q1_p50_us | 169.8 | 44.5 | 176.3 | 42.4 | 43.5 | 13.9 | +3.8% | 0.15 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2734 | 205.2 | 2300 | 360.8 | 293.5 | 333.7 | -15.9% | 1.48 | no difference |
| P vs M | c4_q2_p50_us | 323.4 | 99.7 | 332.2 | 34.1 | 74.5 | 75.9 | +2.7% | 0.12 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 4165 | 917.2 | 2942 | 686.5 | 810.1 | 3809 | -29.4% | 1.51 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 176800 | 3105 | 171840 | 1638 | 2483 | 2227 | -2.8% | 2.00 | no difference |
| P vs M | write_p99_us | 208524 | 16662 | 213054 | 7047 | 12792 | 7418 | +2.2% | 0.35 | BELOW FLOOR |
| P vs M | commits_per_s | 5.51 | 0.11 | 5.68 | 0.03 | 0.08 | 0.09 | +2.9% | 2.05 | no difference |
| P vs M | pss_mib | 45.3 | 0.05 | 45.5 | 0.07 | 0.06 | 5.51 | +0.6% | 4.29 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -785.4 | 187.3 | -815.6 | 161.7 | 175.0 | 4526 | +3.8% | 0.17 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 209.7 | 11.0 | 523.7 | 33.2 | 24.8 | 35.2 | +149.7% | 12.7 | **N better** |
| N vs P+ | q1_p99_us | 1377 | 478.7 | 4662 | 521.4 | 500.5 | 1679 | +238.6% | 6.56 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q2_p50_us | 649981 | 111023 | 438.1 | 13.7 | 78505 | 118548 | -99.9% | 8.27 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 117774 | 10547 | 132653 | 5864 | 8533 | 123385 | +12.6% | 1.74 | BELOW FLOOR |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 139.3 | 13.5 | 528.6 | 52.2 | 38.1 | 22.8 | +279.6% | 10.2 | **N better** |
| N vs P+ | c2_q1_p99_us | 1381 | 823.8 | 4235 | 371.1 | 638.9 | 821.6 | +206.7% | 4.47 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 671548 | 72177 | 505.0 | 28.1 | 51037 | 9781 | -99.9% | 13.1 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 132.2 | 21.6 | 1472 | 40.5 | 32.5 | 44.9 | +1013.4% | 41.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 6060 | 493.8 | 7963 | 918.9 | 737.6 | 2659 | +31.4% | 2.58 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 1231696 | 76656 | 1353 | 76.6 | 54204 | 22138 | -99.9% | 22.7 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 2643605 | 67386 | 7015 | 1742 | 47665 | 89661 | -99.7% | 55.3 | **P+ better** |
| N vs P+ | write_p50_us | 804.7 | 51.2 | 1075 | 74.2 | 63.7 | 306.6 | +33.6% | 4.24 | BELOW FLOOR |
| N vs P+ | write_p99_us | 1910 | 377.1 | 3467 | 621.5 | 514.0 | 6222 | +81.5% | 3.03 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1080 | 85.5 | 750.2 | 50.8 | 70.4 | 110.8 | -30.6% | 4.69 | **N better** |
| N vs P+ | pss_mib | 2545 | 3.12 | 493.8 | 1.11 | 2.34 | 113.0 | -80.6% | 876.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 590394 | 144490 | 52259 | 13201 | 102596 | 1109328 | -91.1% | 5.25 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 209.7 | 11.0 | 148.0 | 5.37 | 8.67 | 10.1 | -29.4% | 7.12 | **P better** |
| N vs P | q1_p99_us | 1377 | 478.7 | 1432 | 656.5 | 574.5 | 748.9 | +4.0% | 0.10 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 649981 | 111023 | 244.7 | 12.6 | 78505 | 118548 | -100.0% | 8.28 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 117774 | 10547 | 212.7 | 26.8 | 7458 | 109127 | -99.8% | 15.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 139.3 | 13.5 | 64.8 | 6.74 | 10.7 | 27.7 | -53.5% | 6.98 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 1381 | 823.8 | 1097 | 142.8 | 591.2 | 1075 | -20.6% | 0.48 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 671548 | 72177 | 187.4 | 28.8 | 51037 | 9781 | -100.0% | 13.2 | **P better** |
| N vs P | c4_q1_p50_us | 132.2 | 21.6 | 172.9 | 15.5 | 18.8 | 23.1 | +30.8% | 2.16 | no difference |
| N vs P | c4_q1_p99_us | 6060 | 493.8 | 2969 | 617.0 | 558.8 | 2659 | -51.0% | 5.53 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 1231696 | 76656 | 346.0 | 41.8 | 54204 | 22137 | -100.0% | 22.7 | **P better** |
| N vs P | c4_q2_p99_us | 2643605 | 67386 | 11016 | 2880 | 47692 | 89661 | -99.6% | 55.2 | **P better** |
| N vs P | write_p50_us | 804.7 | 51.2 | 2611302 | 83191 | 58825 | 114123 | +324396.7% | 44.4 | **N better** |
| N vs P | write_p99_us | 1910 | 377.1 | 2780263 | 67193 | 47514 | 6048 | +145466.7% | 58.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1080 | 85.5 | 0.38 | 0.01 | 60.5 | 51.3 | -100.0% | 17.9 | **N better** |
| N vs P | pss_mib | 2545 | 3.12 | 489.5 | 2.21 | 2.70 | 112.3 | -80.8% | 760.3 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 590394 | 144490 | 53515 | 10208 | 102425 | 1110781 | -90.9% | 5.24 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 523.7 | 33.2 | 148.0 | 5.37 | 23.8 | 33.9 | -71.7% | 15.8 | **P better** |
| P+ vs P | q1_p99_us | 4662 | 521.4 | 1432 | 656.5 | 592.8 | 1694 | -69.3% | 5.45 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 438.1 | 13.7 | 244.7 | 12.6 | 13.2 | 61.3 | -44.1% | 14.7 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 284.0 | 39.0 | 261.4 | 24.4 | 32.5 | 69.9 | -8.0% | 0.69 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 89811 | 3209 | 7796 | 84.1 | 2270 | 4902 | -91.3% | 36.1 | **P better** |
| P+ vs P | q5_p50_us | 132653 | 5864 | 212.7 | 26.8 | 4147 | 57579 | -99.8% | 31.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 528.6 | 52.2 | 64.8 | 6.74 | 37.2 | 35.8 | -87.7% | 12.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 4235 | 371.1 | 1097 | 142.8 | 281.2 | 723.1 | -74.1% | 11.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 505.0 | 28.1 | 187.4 | 28.8 | 28.4 | 11.3 | -62.9% | 11.2 | **P better** |
| P+ vs P | c4_q1_p50_us | 1472 | 40.5 | 172.9 | 15.5 | 30.7 | 50.3 | -88.3% | 42.4 | **P better** |
| P+ vs P | c4_q1_p99_us | 7963 | 918.9 | 2969 | 617.0 | 782.6 | 3438 | -62.7% | 6.38 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1353 | 76.6 | 346.0 | 41.8 | 61.7 | 288.4 | -74.4% | 16.3 | **P better** |
| P+ vs P | c4_q2_p99_us | 7015 | 1742 | 11016 | 2880 | 2380 | 1210 | +57.0% | 1.68 | no difference |
| P+ vs P | write_p50_us | 1075 | 74.2 | 2611302 | 83191 | 58825 | 114122 | +242801.7% | 44.4 | **P+ better** |
| P+ vs P | write_p99_us | 3467 | 621.5 | 2780263 | 67193 | 47515 | 4216 | +80084.7% | 58.4 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 750.2 | 50.8 | 0.38 | 0.01 | 35.9 | 98.2 | -99.9% | 20.9 | **P+ better** |
| P+ vs P | pss_mib | 493.8 | 1.11 | 489.5 | 2.21 | 1.75 | 16.8 | -0.9% | 2.43 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | 52259 | 13201 | 53515 | 10208 | 11800 | 194402 | +2.4% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 148.0 | 5.37 | 145.1 | 4.70 | 5.05 | 5.28 | -2.0% | 0.58 | BELOW FLOOR |
| P vs M | q1_p99_us | 1432 | 656.5 | 1170 | 630.4 | 643.6 | 708.1 | -18.3% | 0.41 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 244.7 | 12.6 | 241.1 | 19.4 | 16.4 | 24.1 | -1.5% | 0.22 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 261.4 | 24.4 | 268.9 | 26.1 | 25.3 | 68.9 | +2.9% | 0.30 | BELOW FLOOR |
| P vs M | q4_p50_us | 7796 | 84.1 | 7593 | 171.7 | 135.2 | 376.1 | -2.6% | 1.51 | BELOW FLOOR |
| P vs M | q5_p50_us | 212.7 | 26.8 | 205.1 | 12.3 | 20.9 | 131.3 | -3.6% | 0.37 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 64.8 | 6.74 | 135.0 | 21.4 | 15.9 | 43.6 | +108.5% | 4.43 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1097 | 142.8 | 2206 | 623.1 | 452.0 | 743.8 | +101.1% | 2.45 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 187.4 | 28.8 | 279.8 | 47.9 | 39.5 | 6.53 | +49.3% | 2.34 | no difference |
| P vs M | c4_q1_p50_us | 172.9 | 15.5 | 197.3 | 40.2 | 30.4 | 39.8 | +14.1% | 0.80 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2969 | 617.0 | 3452 | 725.0 | 673.2 | 2437 | +16.3% | 0.72 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 346.0 | 41.8 | 358.8 | 20.5 | 32.9 | 52.7 | +3.7% | 0.39 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 11016 | 2880 | 13323 | 3794 | 3368 | 4740 | +21.0% | 0.69 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 2611302 | 83191 | 2642827 | 45697 | 67115 | 115722 | +1.2% | 0.47 | BELOW FLOOR |
| P vs M | write_p99_us | 2780263 | 67193 | 2856335 | 78939 | 73302 | 19545 | +2.7% | 1.04 | no difference |
| P vs M | commits_per_s | 0.38 | 0.01 | 0.38 | 0.01 | 0.01 | 0.01 | -2.0% | 0.81 | BELOW FLOOR |
| P vs M | pss_mib | 489.5 | 2.21 | 488.9 | 1.47 | 1.88 | 10.5 | -0.1% | 0.32 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 53515 | 10208 | 48807 | 19687 | 15681 | 196653 | -8.8% | 0.30 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 229.9 | 12.4 | 565.1 | 18.5 | 15.7 | 89.7 | +145.8% | 21.3 | **N better** |
| N vs P+ | q1_p99_us | 2436 | 1347 | 3771 | 986.4 | 1181 | 1414 | +54.8% | 1.13 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 735036 | 60556 | 461.7 | 32.8 | 42819 | 57467 | -99.9% | 17.2 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 374381 | 267500 | 559362 | 424963 | 355070 | 84266 | +49.4% | 0.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q6_p50_us | 1473453 | 60469 | 2065526 | 60796 | 60633 | 150636 | +40.2% | 9.76 | **N better** |
| N vs P+ | c2_q1_p50_us | 135.3 | 14.6 | 540.5 | 30.1 | 23.7 | 28.8 | +299.4% | 17.1 | **N better** |
| N vs P+ | c2_q1_p99_us | 927.6 | 413.9 | 5578 | 1111 | 838.6 | 6644 | +501.3% | 5.55 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 739118 | 71281 | 502.8 | 33.1 | 50403 | 269714 | -99.9% | 14.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 130.8 | 15.4 | 1464 | 63.4 | 46.1 | 138.6 | +1019.9% | 28.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 5649 | 179.6 | 7468 | 1595 | 1135 | 1057 | +32.2% | 1.60 | no difference |
| N vs P+ | c4_q2_p50_us | 1192228 | 56213 | 1370 | 87.9 | 39749 | 521546 | -99.9% | 30.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 2912471 | 185105 | 9572 | 1672 | 130894 | 301344 | -99.7% | 22.2 | **P+ better** |
| N vs P+ | write_p50_us | 856.4 | 51.2 | 1066 | 45.4 | 48.4 | 198.7 | +24.5% | 4.33 | **N better** |
| N vs P+ | write_p99_us | 2866 | 864.6 | 3422 | 157.5 | 621.4 | 6288 | +19.4% | 0.90 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | commits_per_s | 895.3 | 71.5 | 759.7 | 70.7 | 71.1 | 317.3 | -15.1% | 1.91 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | pss_mib | 2195 | 484.3 | 495.8 | 1.32 | 342.4 | 31.3 | -77.4% | 4.96 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 519792 | 74873 | 55269 | 16759 | 54253 | 994641 | -89.4% | 8.56 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 229.9 | 12.4 | 149.1 | 4.47 | 9.35 | 14.0 | -35.1% | 8.64 | **P better** |
| N vs P | q1_p99_us | 2436 | 1347 | 1315 | 553.3 | 1030 | 1295 | -46.0% | 1.09 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 735036 | 60556 | 234.5 | 13.1 | 42819 | 57467 | -100.0% | 17.2 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 374381 | 267500 | 204.1 | 24.5 | 189151 | 83700 | -99.9% | 1.98 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q6_p50_us | 1473453 | 60469 | 39661 | 1859 | 42778 | 150422 | -97.3% | 33.5 | **P better** |
| N vs P | c2_q1_p50_us | 135.3 | 14.6 | 92.5 | 33.0 | 25.5 | 69.5 | -31.6% | 1.68 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 927.6 | 413.9 | 2119 | 573.4 | 500.0 | 450.0 | +128.4% | 2.38 | no difference |
| N vs P | c2_q2_p50_us | 739118 | 71281 | 220.7 | 47.0 | 50403 | 269714 | -100.0% | 14.7 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p50_us | 130.8 | 15.4 | 184.0 | 24.5 | 20.5 | 49.9 | +40.7% | 2.60 | no difference |
| N vs P | c4_q1_p99_us | 5649 | 179.6 | 3466 | 764.3 | 555.2 | 734.9 | -38.6% | 3.93 | **P better** |
| N vs P | c4_q2_p50_us | 1192228 | 56213 | 337.0 | 39.5 | 39749 | 521546 | -100.0% | 30.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 2912471 | 185105 | 8723 | 4398 | 130926 | 301367 | -99.7% | 22.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 856.4 | 51.2 | 2687973 | 97161 | 68703 | 29381 | +313769.4% | 39.1 | **N better** |
| N vs P | write_p99_us | 2866 | 864.6 | 2872813 | 84624 | 59841 | 71921 | +100134.2% | 48.0 | **N better** |
| N vs P | commits_per_s | 895.3 | 71.5 | 0.37 | 0.01 | 50.6 | 191.1 | -100.0% | 17.7 | **N better** |
| N vs P | pss_mib | 2195 | 484.3 | 490.8 | 1.77 | 342.4 | 31.2 | -77.6% | 4.98 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 519792 | 74873 | 43899 | 21088 | 55003 | 997774 | -91.6% | 8.65 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 565.1 | 18.5 | 149.1 | 4.47 | 13.4 | 89.8 | -73.6% | 31.0 | **P better** |
| P+ vs P | q1_p99_us | 3771 | 986.4 | 1315 | 553.3 | 799.7 | 914.4 | -65.1% | 3.07 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 461.7 | 32.8 | 234.5 | 13.1 | 25.0 | 26.2 | -49.2% | 9.10 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 261.1 | 15.0 | 244.4 | 17.0 | 16.0 | 77.8 | -6.4% | 1.04 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 90768 | 3052 | 7548 | 145.0 | 2160 | 5780 | -91.7% | 38.5 | **P better** |
| P+ vs P | q5_p50_us | 559362 | 424963 | 204.1 | 24.5 | 300494 | 9751 | -100.0% | 1.86 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 2065526 | 60796 | 39661 | 1859 | 43010 | 8015 | -98.1% | 47.1 | **P better** |
| P+ vs P | c2_q1_p50_us | 540.5 | 30.1 | 92.5 | 33.0 | 31.6 | 63.8 | -82.9% | 14.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5578 | 1111 | 2119 | 573.4 | 884.3 | 6635 | -62.0% | 3.91 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 502.8 | 33.1 | 220.7 | 47.0 | 40.6 | 93.2 | -56.1% | 6.94 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1464 | 63.4 | 184.0 | 24.5 | 48.0 | 146.9 | -87.4% | 26.6 | **P better** |
| P+ vs P | c4_q1_p99_us | 7468 | 1595 | 3466 | 764.3 | 1251 | 1271 | -53.6% | 3.20 | **P better** |
| P+ vs P | c4_q2_p50_us | 1370 | 87.9 | 337.0 | 39.5 | 68.2 | 34.6 | -75.4% | 15.2 | **P better** |
| P+ vs P | c4_q2_p99_us | 9572 | 1672 | 8723 | 4398 | 3327 | 3934 | -8.9% | 0.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 1066 | 45.4 | 2687973 | 97161 | 68703 | 29382 | +252081.3% | 39.1 | **P+ better** |
| P+ vs P | write_p99_us | 3422 | 157.5 | 2872813 | 84624 | 59838 | 72188 | +83840.2% | 48.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 759.7 | 70.7 | 0.37 | 0.01 | 50.0 | 253.3 | -100.0% | 15.2 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | pss_mib | 495.8 | 1.32 | 490.8 | 1.77 | 1.56 | 3.32 | -1.0% | 3.22 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 55269 | 16759 | 43899 | 21088 | 19047 | 186241 | -20.6% | 0.60 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 149.1 | 4.47 | 151.1 | 3.53 | 4.03 | 10.9 | +1.4% | 0.50 | BELOW FLOOR |
| P vs M | q1_p99_us | 1315 | 553.3 | 1866 | 760.5 | 665.0 | 541.7 | +41.8% | 0.83 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 234.5 | 13.1 | 242.1 | 18.3 | 15.9 | 38.4 | +3.3% | 0.48 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 244.4 | 17.0 | 231.7 | 16.0 | 16.5 | 76.0 | -5.2% | 0.77 | BELOW FLOOR |
| P vs M | q4_p50_us | 7548 | 145.0 | 7641 | 103.7 | 126.0 | 924.9 | +1.2% | 0.74 | BELOW FLOOR |
| P vs M | q5_p50_us | 204.1 | 24.5 | 202.7 | 24.2 | 24.4 | 286.2 | -0.7% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q6_p50_us | 39661 | 1859 | 39142 | 796.2 | 1430 | 2440 | -1.3% | 0.36 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 92.5 | 33.0 | 62.6 | 3.44 | 23.4 | 64.0 | -32.4% | 1.28 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 2119 | 573.4 | 1125 | 408.2 | 497.7 | 309.3 | -46.9% | 2.00 | no difference |
| P vs M | c2_q2_p50_us | 220.7 | 47.0 | 185.6 | 26.6 | 38.2 | 103.8 | -15.9% | 0.92 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 184.0 | 24.5 | 164.8 | 19.4 | 22.1 | 53.9 | -10.4% | 0.87 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 3466 | 764.3 | 3573 | 816.1 | 790.6 | 1675 | +3.1% | 0.14 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 337.0 | 39.5 | 326.7 | 35.8 | 37.7 | 105.6 | -3.0% | 0.27 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 8723 | 4398 | 10999 | 5886 | 5196 | 3855 | +26.1% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 2687973 | 97161 | 2649167 | 68315 | 83986 | 144789 | -1.4% | 0.46 | BELOW FLOOR |
| P vs M | write_p99_us | 2872813 | 84624 | 2897509 | 49389 | 69284 | 137082 | +0.9% | 0.36 | BELOW FLOOR |
| P vs M | commits_per_s | 0.37 | 0.01 | 0.38 | 0.01 | 0.01 | 0.01 | +1.7% | 0.52 | BELOW FLOOR |
| P vs M | pss_mib | 490.8 | 1.77 | 489.2 | 1.60 | 1.69 | 11.0 | -0.3% | 0.98 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 43899 | 21088 | 51853 | 17546 | 19398 | 195066 | +18.1% | 0.41 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 237.1 | 11.4 | 576.3 | 36.8 | 27.3 | 37.3 | +143.1% | 12.4 | **N better** |
| N vs P+ | q1_p99_us | 1243 | 446.9 | 4273 | 372.2 | 411.3 | 577.3 | +243.7% | 7.37 | **N better** |
| N vs P+ | q2_p50_us | 736492 | 77480 | 478.6 | 32.3 | 54787 | 232419 | -99.9% | 13.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 121252 | 8918 | 165521 | 28107 | 20851 | 197211 | +36.5% | 2.12 | BELOW FLOOR |
| N vs P+ | q6_p50_us | 1494609 | 50255 | 2164860 | 69473 | 60630 | 41677 | +44.8% | 11.1 | **N better** |
| N vs P+ | c2_q1_p50_us | 132.0 | 11.2 | 570.4 | 61.8 | 44.4 | 66.1 | +332.0% | 9.87 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 954.0 | 569.1 | 5247 | 822.0 | 706.9 | 1083 | +450.0% | 6.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 730683 | 62853 | 524.0 | 60.3 | 44444 | 33278 | -99.9% | 16.4 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 135.2 | 10.1 | 1562 | 70.5 | 50.4 | 260.7 | +1055.8% | 28.3 | **N better** |
| N vs P+ | c4_q1_p99_us | 5937 | 184.2 | 7310 | 1720 | 1223 | 5758 | +23.1% | 1.12 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 1218870 | 128225 | 1468 | 86.8 | 90669 | 333201 | -99.9% | 13.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 2922993 | 240702 | 7228 | 2158 | 170209 | 111097 | -99.8% | 17.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 905.3 | 64.9 | 1090 | 93.3 | 80.3 | 382.9 | +20.4% | 2.30 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 3441 | 511.0 | 3583 | 592.6 | 553.3 | 2755 | +4.1% | 0.26 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 899.2 | 82.6 | 704.7 | 27.4 | 61.5 | 440.9 | -21.6% | 3.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 1708 | 0.82 | 494.9 | 0.65 | 0.74 | 2.93 | -71.0% | 1642 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 374021 | 78749 | 54358 | 13859 | 56540 | 158402 | -85.5% | 5.65 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 237.1 | 11.4 | 154.1 | 3.57 | 8.47 | 48.4 | -35.0% | 9.80 | **P better** |
| N vs P | q1_p99_us | 1243 | 446.9 | 1453 | 831.0 | 667.2 | 802.1 | +16.9% | 0.31 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 736492 | 77480 | 258.5 | 18.6 | 54787 | 232419 | -100.0% | 13.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 121252 | 8918 | 231.4 | 47.4 | 6306 | 186767 | -99.8% | 19.2 | BELOW FLOOR |
| N vs P | q6_p50_us | 1494609 | 50255 | 39502 | 981.4 | 35542 | 42026 | -97.4% | 40.9 | **P better** |
| N vs P | c2_q1_p50_us | 132.0 | 11.2 | 62.0 | 1.74 | 8.01 | 66.1 | -53.1% | 8.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 954.0 | 569.1 | 1257 | 409.2 | 495.6 | 300.1 | +31.8% | 0.61 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 730683 | 62853 | 156.8 | 14.5 | 44444 | 33278 | -100.0% | 16.4 | **P better** |
| N vs P | c4_q1_p50_us | 135.2 | 10.1 | 140.3 | 43.7 | 31.7 | 42.5 | +3.8% | 0.16 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 5937 | 184.2 | 3486 | 670.0 | 491.3 | 976.9 | -41.3% | 4.99 | **P better** |
| N vs P | c4_q2_p50_us | 1218870 | 128225 | 252.9 | 49.0 | 90669 | 333201 | -100.0% | 13.4 | **P better** |
| N vs P | c4_q2_p99_us | 2922993 | 240702 | 11579 | 8793 | 170316 | 111664 | -99.6% | 17.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 905.3 | 64.9 | 2846681 | 110740 | 78305 | 76430 | +314335.2% | 36.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p99_us | 3441 | 511.0 | 3105803 | 186336 | 131760 | 64538 | +90146.6% | 23.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 899.2 | 82.6 | 0.35 | 0.02 | 58.4 | 437.6 | -100.0% | 15.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_mib | 1708 | 0.82 | 491.8 | 1.46 | 1.19 | 16.6 | -71.2% | 1026 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 374021 | 78749 | 54376 | 13176 | 56458 | 215296 | -85.5% | 5.66 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 576.3 | 36.8 | 154.1 | 3.57 | 26.2 | 34.0 | -73.3% | 16.1 | **P better** |
| P+ vs P | q1_p99_us | 4273 | 372.2 | 1453 | 831.0 | 643.9 | 988.0 | -66.0% | 4.38 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 478.6 | 32.3 | 258.5 | 18.6 | 26.3 | 66.7 | -46.0% | 8.36 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 261.8 | 8.91 | 268.3 | 32.8 | 24.0 | 51.2 | +2.5% | 0.27 | BELOW FLOOR |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q5_p50_us | 165521 | 28107 | 231.4 | 47.4 | 19875 | 63326 | -99.9% | 8.32 | **P better** |
| P+ vs P | q6_p50_us | 2164860 | 69473 | 39502 | 981.4 | 49130 | 12679 | -98.2% | 43.3 | **P better** |
| P+ vs P | c2_q1_p50_us | 570.4 | 61.8 | 62.0 | 1.74 | 43.7 | 10.8 | -89.1% | 11.6 | **P better** |
| P+ vs P | c2_q1_p99_us | 5247 | 822.0 | 1257 | 409.2 | 649.3 | 1091 | -76.0% | 6.15 | **P better** |
| P+ vs P | c2_q2_p50_us | 524.0 | 60.3 | 156.8 | 14.5 | 43.9 | 41.8 | -70.1% | 8.37 | **P better** |
| P+ vs P | c4_q1_p50_us | 1562 | 70.5 | 140.3 | 43.7 | 58.7 | 261.0 | -91.0% | 24.2 | **P better** |
| P+ vs P | c4_q1_p99_us | 7310 | 1720 | 3486 | 670.0 | 1305 | 5836 | -52.3% | 2.93 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1468 | 86.8 | 252.9 | 49.0 | 70.5 | 31.6 | -82.8% | 17.2 | **P better** |
| P+ vs P | c4_q2_p99_us | 7228 | 2158 | 11579 | 8793 | 6402 | 11773 | +60.2% | 0.68 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 1090 | 93.3 | 2846681 | 110740 | 78305 | 76430 | +261018.3% | 36.3 | **P+ better** |
| P+ vs P | write_p99_us | 3583 | 592.6 | 3105803 | 186336 | 131760 | 64533 | +86573.8% | 23.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 704.7 | 27.4 | 0.35 | 0.02 | 19.4 | 54.1 | -100.0% | 36.4 | **P+ better** |
| P+ vs P | pss_mib | 494.9 | 0.65 | 491.8 | 1.46 | 1.13 | 16.9 | -0.6% | 2.79 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | 54358 | 13859 | 54376 | 13176 | 13522 | 261997 | +0.0% | 0.00 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 154.1 | 3.57 | 151.1 | 8.14 | 6.28 | 32.9 | -2.0% | 0.48 | BELOW FLOOR |
| P vs M | q1_p99_us | 1453 | 831.0 | 1104 | 586.9 | 719.4 | 980.1 | -24.0% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 258.5 | 18.6 | 262.9 | 27.2 | 23.3 | 63.1 | +1.7% | 0.19 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 268.3 | 32.8 | 250.6 | 18.3 | 26.5 | 49.6 | -6.6% | 0.67 | BELOW FLOOR |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q5_p50_us | 231.4 | 47.4 | 210.5 | 37.7 | 42.8 | 54.3 | -9.0% | 0.49 | BELOW FLOOR |
| P vs M | q6_p50_us | 39502 | 981.4 | 41890 | 5316 | 3822 | 9784 | +6.0% | 0.62 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 62.0 | 1.74 | 93.9 | 35.0 | 24.8 | 42.2 | +51.6% | 1.29 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1257 | 409.2 | 1559 | 345.6 | 378.8 | 424.5 | +24.0% | 0.80 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 156.8 | 14.5 | 216.7 | 49.1 | 36.2 | 138.1 | +38.3% | 1.66 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 140.3 | 43.7 | 192.3 | 31.6 | 38.1 | 117.5 | +37.0% | 1.36 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 3486 | 670.0 | 2874 | 667.8 | 668.9 | 1497 | -17.6% | 0.91 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 252.9 | 49.0 | 358.9 | 99.2 | 78.2 | 72.3 | +41.9% | 1.35 | no difference |
| P vs M | c4_q2_p99_us | 11579 | 8793 | 13027 | 9243 | 9021 | 32908 | +12.5% | 0.16 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 2846681 | 110740 | 2825880 | 54418 | 87248 | 94059 | -0.7% | 0.24 | BELOW FLOOR |
| P vs M | write_p99_us | 3105803 | 186336 | 3124935 | 71713 | 141180 | 69924 | +0.6% | 0.14 | BELOW FLOOR |
| P vs M | commits_per_s | 0.35 | 0.02 | 0.35 | 0.01 | 0.01 | 0.01 | +1.0% | 0.28 | BELOW FLOOR |
| P vs M | pss_mib | 491.8 | 1.46 | 489.7 | 3.60 | 2.74 | 18.1 | -0.4% | 0.74 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 54376 | 13176 | 49030 | 14354 | 13778 | 254362 | -9.8% | 0.39 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 220.1 | 17.6 | 584.6 | 45.4 | 34.4 | 104.9 | +165.6% | 10.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q1_p99_us | 2174 | 975.7 | 4915 | 407.2 | 747.6 | 798.9 | +126.1% | 3.67 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 694033 | 60079 | 511.3 | 28.3 | 42482 | 59895 | -99.9% | 16.3 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 661444 | 57113 | 1022507 | 155826 | 117353 | 151133 | +54.6% | 3.08 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 122.7 | 11.6 | 559.7 | 28.1 | 21.5 | 232.3 | +356.1% | 20.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q1_p99_us | 850.3 | 348.9 | 5230 | 296.2 | 323.6 | 1218 | +515.1% | 13.5 | **N better** |
| N vs P+ | c2_q2_p50_us | 723330 | 44637 | 519.3 | 28.6 | 31563 | 39626 | -99.9% | 22.9 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q1_p50_us | 133.2 | 6.39 | 1711 | 115.4 | 81.7 | 170.0 | +1184.5% | 19.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 6062 | 556.5 | 7942 | 1042 | 835.2 | 1838 | +31.0% | 2.25 | no difference |
| N vs P+ | c4_q2_p50_us | 1166829 | 106735 | 1473 | 116.0 | 75473 | 8084 | -99.9% | 15.4 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 2744579 | 98339 | 7112 | 1621 | 69546 | 495730 | -99.7% | 39.4 | **P+ better** |
| N vs P+ | write_p50_us | 890.0 | 29.9 | 1140 | 117.1 | 85.5 | 669.4 | +28.1% | 2.93 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p99_us | 2574 | 325.5 | 4223 | 626.9 | 499.5 | 201.0 | +64.0% | 3.30 | **N better** |
| N vs P+ | commits_per_s | 956.0 | 90.4 | 694.7 | 27.5 | 66.8 | 461.1 | -27.3% | 3.91 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | pss_mib | 2117 | 63.9 | 497.1 | 0.28 | 45.2 | 155.9 | -76.5% | 35.9 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 439009 | 65522 | 54805 | 13953 | 47370 | 982527 | -87.5% | 8.11 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 220.1 | 17.6 | 163.2 | 11.0 | 14.7 | 74.7 | -25.8% | 3.87 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q1_p99_us | 2174 | 975.7 | 2549 | 989.9 | 982.8 | 710.3 | +17.2% | 0.38 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 694033 | 60079 | 259.5 | 20.6 | 42482 | 59895 | -100.0% | 16.3 | **P better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 661444 | 57113 | 212.1 | 35.0 | 40385 | 95769 | -100.0% | 16.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 122.7 | 11.6 | 123.1 | 18.8 | 15.6 | 20.7 | +0.4% | 0.03 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 850.3 | 348.9 | 2037 | 724.7 | 568.7 | 1187 | +139.5% | 2.09 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 723330 | 44637 | 212.3 | 64.3 | 31563 | 39626 | -100.0% | 22.9 | **P better** |
| N vs P | c4_q1_p50_us | 133.2 | 6.39 | 201.6 | 39.3 | 28.2 | 90.3 | +51.3% | 2.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p99_us | 6062 | 556.5 | 3536 | 503.9 | 530.9 | 2221 | -41.7% | 4.76 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 1166829 | 106735 | 377.7 | 31.3 | 75473 | 8079 | -100.0% | 15.5 | **P better** |
| N vs P | c4_q2_p99_us | 2744579 | 98339 | 8838 | 3537 | 69581 | 495888 | -99.7% | 39.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 890.0 | 29.9 | 2787719 | 59121 | 41805 | 25150 | +313143.5% | 66.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p99_us | 2574 | 325.5 | 3187164 | 162900 | 115188 | 338097 | +123707.4% | 27.6 | **N better** |
| N vs P | commits_per_s | 956.0 | 90.4 | 0.35 | 0.01 | 63.9 | 326.2 | -100.0% | 15.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_mib | 2117 | 63.9 | 490.7 | 2.09 | 45.2 | 155.9 | -76.8% | 36.0 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 439009 | 65522 | 53791 | 12934 | 47225 | 985284 | -87.7% | 8.16 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 584.6 | 45.4 | 163.2 | 11.0 | 33.0 | 77.2 | -72.1% | 12.8 | **P better** |
| P+ vs P | q1_p99_us | 4915 | 407.2 | 2549 | 989.9 | 756.9 | 448.4 | -48.2% | 3.13 | **P better** |
| P+ vs P | q2_p50_us | 511.3 | 28.3 | 259.5 | 20.6 | 24.8 | 33.1 | -49.2% | 10.2 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 302.3 | 42.2 | 274.3 | 16.8 | 32.1 | 191.4 | -9.3% | 0.87 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q4_p50_us | 93117 | 1708 | 7825 | 176.4 | 1215 | 15871 | -91.6% | 70.2 | **P better** |
| P+ vs P | q5_p50_us | 1022507 | 155826 | 212.1 | 35.0 | 110186 | 116917 | -100.0% | 9.28 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 559.7 | 28.1 | 123.1 | 18.8 | 23.9 | 232.1 | -78.0% | 18.3 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q1_p99_us | 5230 | 296.2 | 2037 | 724.7 | 553.6 | 1694 | -61.1% | 5.77 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 519.3 | 28.6 | 212.3 | 64.3 | 49.7 | 216.9 | -59.1% | 6.17 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q1_p50_us | 1711 | 115.4 | 201.6 | 39.3 | 86.2 | 144.1 | -88.2% | 17.5 | **P better** |
| P+ vs P | c4_q1_p99_us | 7942 | 1042 | 3536 | 503.9 | 818.3 | 2540 | -55.5% | 5.39 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1473 | 116.0 | 377.7 | 31.3 | 85.0 | 278.4 | -74.4% | 12.9 | **P better** |
| P+ vs P | c4_q2_p99_us | 7112 | 1621 | 8838 | 3537 | 2751 | 12587 | +24.3% | 0.63 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 1140 | 117.1 | 2787719 | 59121 | 41805 | 25150 | +244405.8% | 66.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p99_us | 4223 | 626.9 | 3187164 | 162900 | 115188 | 338097 | +75379.1% | 27.6 | **P+ better** |
| P+ vs P | commits_per_s | 694.7 | 27.5 | 0.35 | 0.01 | 19.4 | 325.8 | -99.9% | 35.7 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | pss_mib | 497.1 | 0.28 | 490.7 | 2.09 | 1.49 | 2.25 | -1.3% | 4.32 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 54805 | 13953 | 53791 | 12934 | 13454 | 192887 | -1.8% | 0.08 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 163.2 | 11.0 | 162.5 | 10.3 | 10.7 | 16.4 | -0.4% | 0.06 | BELOW FLOOR |
| P vs M | q1_p99_us | 2549 | 989.9 | 1634 | 715.0 | 863.4 | 863.4 | -35.9% | 1.06 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 259.5 | 20.6 | 261.6 | 39.7 | 31.6 | 12.4 | +0.8% | 0.07 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 274.3 | 16.8 | 267.0 | 35.5 | 27.8 | 168.6 | -2.7% | 0.26 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 7825 | 176.4 | 7777 | 166.6 | 171.6 | 854.9 | -0.6% | 0.28 | BELOW FLOOR |
| P vs M | q5_p50_us | 212.1 | 35.0 | 227.5 | 26.7 | 31.1 | 75.6 | +7.3% | 0.49 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 123.1 | 18.8 | 101.1 | 31.8 | 26.1 | 39.8 | -17.9% | 0.84 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 2037 | 724.7 | 1288 | 154.4 | 523.9 | 1255 | -36.7% | 1.43 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 212.3 | 64.3 | 203.8 | 27.0 | 49.3 | 136.0 | -4.0% | 0.17 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 201.6 | 39.3 | 166.5 | 11.1 | 28.9 | 13.9 | -17.4% | 1.21 | no difference |
| P vs M | c4_q1_p99_us | 3536 | 503.9 | 3723 | 983.0 | 781.1 | 2057 | +5.3% | 0.24 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 377.7 | 31.3 | 316.8 | 21.4 | 26.8 | 94.7 | -16.1% | 2.27 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 8838 | 3537 | 7592 | 1725 | 2783 | 17539 | -14.1% | 0.45 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 2787719 | 59121 | 2728370 | 72777 | 66301 | 359367 | -2.1% | 0.90 | BELOW FLOOR |
| P vs M | write_p99_us | 3187164 | 162900 | 3041532 | 95624 | 133567 | 380880 | -4.6% | 1.09 | BELOW FLOOR |
| P vs M | commits_per_s | 0.35 | 0.01 | 0.37 | 0.01 | 0.01 | 0.04 | +2.9% | 0.96 | BELOW FLOOR |
| P vs M | pss_mib | 490.7 | 2.09 | 490.4 | 0.62 | 1.54 | 12.6 | -0.1% | 0.20 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 53791 | 12934 | 54002 | 12815 | 12875 | 196795 | +0.4% | 0.02 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `multi`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 229.3 | 24.7 | 566.4 | 25.9 | 25.3 | 10.6 | +147.0% | 13.3 | **N better** |
| N vs P+ | q1_p99_us | 2516 | 876.3 | 3714 | 855.5 | 866.0 | 2136 | +47.6% | 1.38 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 744400 | 93650 | 474.6 | 6.70 | 66221 | 46777 | -99.9% | 11.2 | **P+ better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 385782 | 274977 | 600087 | 470714 | 385476 | 11172 | +55.6% | 0.56 | no difference |
| N vs P+ | q6_p50_us | 1519409 | 65037 | 2031173 | 63412 | 64230 | 360300 | +33.7% | 7.97 | **N better** |
| N vs P+ | c2_q1_p50_us | 154.7 | 27.5 | 569.8 | 32.4 | 30.1 | 121.7 | +268.4% | 13.8 | **N better** |
| N vs P+ | c2_q1_p99_us | 1134 | 575.0 | 5217 | 977.9 | 802.1 | 1250 | +359.9% | 5.09 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 692853 | 46244 | 507.5 | 29.3 | 32699 | 101510 | -99.9% | 21.2 | **P+ better** |
| N vs P+ | c4_q1_p50_us | 142.8 | 15.6 | 1551 | 52.3 | 38.6 | 21.6 | +986.2% | 36.5 | **N better** |
| N vs P+ | c4_q1_p99_us | 5682 | 324.0 | 8699 | 974.4 | 726.1 | 4536 | +53.1% | 4.16 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 1175073 | 56702 | 1499 | 34.2 | 40094 | 229530 | -99.9% | 29.3 | **P+ better** |
| N vs P+ | c4_q2_p99_us | 2779825 | 132401 | 8837 | 2006 | 93633 | 321418 | -99.7% | 29.6 | **P+ better** |
| N vs P+ | write_p50_us | 868.0 | 52.7 | 1086 | 66.7 | 60.1 | 196.9 | +25.2% | 3.63 | **N better** |
| N vs P+ | write_p99_us | 2781 | 524.0 | 3591 | 336.8 | 440.5 | 1250 | +29.1% | 1.84 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 952.5 | 75.2 | 747.3 | 24.4 | 55.9 | 83.0 | -21.5% | 3.67 | **N better** |
| N vs P+ | pss_mib | 2162 | 272.0 | 494.0 | 1.27 | 192.3 | 6.24 | -77.2% | 8.67 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 466013 | 46172 | 55664 | 13928 | 34102 | 184407 | -88.1% | 12.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P | q1_p50_us | 229.3 | 24.7 | 156.7 | 8.68 | 18.5 | 69.5 | -31.7% | 3.92 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q1_p99_us | 2516 | 876.3 | 1660 | 906.7 | 891.7 | 177.5 | -34.0% | 0.96 | no difference |
| N vs P | q2_p50_us | 744400 | 93650 | 243.5 | 10.4 | 66221 | 46777 | -100.0% | 11.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 385782 | 274977 | 246.3 | 24.6 | 194438 | 4912 | -99.9% | 1.98 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 1519409 | 65037 | 39699 | 1406 | 45999 | 95724 | -97.4% | 32.2 | **P better** |
| N vs P | c2_q1_p50_us | 154.7 | 27.5 | 115.6 | 34.6 | 31.3 | 31.6 | -25.3% | 1.25 | no difference |
| N vs P | c2_q1_p99_us | 1134 | 575.0 | 1846 | 565.4 | 570.2 | 969.4 | +62.8% | 1.25 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 692853 | 46244 | 203.6 | 44.8 | 32699 | 101510 | -100.0% | 21.2 | **P better** |
| N vs P | c4_q1_p50_us | 142.8 | 15.6 | 160.1 | 31.3 | 24.7 | 15.6 | +12.1% | 0.70 | no difference |
| N vs P | c4_q1_p99_us | 5682 | 324.0 | 3031 | 719.2 | 557.8 | 842.1 | -46.7% | 4.75 | **P better** |
| N vs P | c4_q2_p50_us | 1175073 | 56702 | 379.2 | 76.2 | 40094 | 229530 | -100.0% | 29.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 2779825 | 132401 | 8658 | 4705 | 93681 | 321493 | -99.7% | 29.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 868.0 | 52.7 | 2794516 | 27288 | 19295 | 4899 | +321851.0% | 144.8 | **N better** |
| N vs P | write_p99_us | 2781 | 524.0 | 3043616 | 54346 | 38430 | 46561 | +109333.0% | 79.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 952.5 | 75.2 | 0.36 | 0.00 | 53.2 | 6.21 | -100.0% | 17.9 | **N better** |
| N vs P | pss_mib | 2162 | 272.0 | 488.5 | 1.26 | 192.3 | 7.72 | -77.4% | 8.70 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 466013 | 46172 | 53326 | 12596 | 33842 | 200092 | -88.6% | 12.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 566.4 | 25.9 | 156.7 | 8.68 | 19.3 | 68.8 | -72.3% | 21.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p99_us | 3714 | 855.5 | 1660 | 906.7 | 881.5 | 2139 | -55.3% | 2.33 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 474.6 | 6.70 | 243.5 | 10.4 | 8.74 | 131.8 | -48.7% | 26.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 291.8 | 20.7 | 294.0 | 28.6 | 24.9 | 1.42 | +0.8% | 0.09 | no difference |
| P+ vs P | q4_p50_us | 91287 | 1829 | 7916 | 220.4 | 1303 | 2889 | -91.3% | 64.0 | **P better** |
| P+ vs P | q5_p50_us | 600087 | 470714 | 246.3 | 24.6 | 332845 | 10035 | -100.0% | 1.80 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 2031173 | 63412 | 39699 | 1406 | 44850 | 347384 | -98.0% | 44.4 | **P better** |
| P+ vs P | c2_q1_p50_us | 569.8 | 32.4 | 115.6 | 34.6 | 33.6 | 125.6 | -79.7% | 13.5 | **P better** |
| P+ vs P | c2_q1_p99_us | 5217 | 977.9 | 1846 | 565.4 | 798.7 | 867.7 | -64.6% | 4.22 | **P better** |
| P+ vs P | c2_q2_p50_us | 507.5 | 29.3 | 203.6 | 44.8 | 37.8 | 44.0 | -59.9% | 8.03 | **P better** |
| P+ vs P | c4_q1_p50_us | 1551 | 52.3 | 160.1 | 31.3 | 43.1 | 15.8 | -89.7% | 32.2 | **P better** |
| P+ vs P | c4_q1_p99_us | 8699 | 974.4 | 3031 | 719.2 | 856.4 | 4463 | -65.2% | 6.62 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1499 | 34.2 | 379.2 | 76.2 | 59.1 | 318.2 | -74.7% | 18.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 8837 | 2006 | 8658 | 4705 | 3617 | 6941 | -2.0% | 0.05 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 1086 | 66.7 | 2794516 | 27288 | 19295 | 4903 | +257121.5% | 144.8 | **P+ better** |
| P+ vs P | write_p99_us | 3591 | 336.8 | 3043616 | 54346 | 38429 | 46549 | +84661.6% | 79.1 | **P+ better** |
| P+ vs P | commits_per_s | 747.3 | 24.4 | 0.36 | 0.00 | 17.2 | 82.8 | -100.0% | 43.3 | **P+ better** |
| P+ vs P | pss_mib | 494.0 | 1.27 | 488.5 | 1.26 | 1.26 | 6.32 | -1.1% | 4.29 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | 55664 | 13928 | 53326 | 12596 | 13279 | 181307 | -4.2% | 0.18 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 156.7 | 8.68 | 155.6 | 5.99 | 7.46 | 70.1 | -0.7% | 0.15 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q1_p99_us | 1660 | 906.7 | 1785 | 540.9 | 746.6 | 220.1 | +7.5% | 0.17 | BELOW FLOOR |
| P vs M | q2_p50_us | 243.5 | 10.4 | 249.6 | 12.9 | 11.7 | 96.1 | +2.5% | 0.53 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 294.0 | 28.6 | 300.9 | 51.5 | 41.6 | 38.1 | +2.3% | 0.17 | BELOW FLOOR |
| P vs M | q4_p50_us | 7916 | 220.4 | 7895 | 352.9 | 294.2 | 396.8 | -0.3% | 0.07 | BELOW FLOOR |
| P vs M | q5_p50_us | 246.3 | 24.6 | 260.8 | 52.2 | 40.8 | 82.1 | +5.9% | 0.35 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 39699 | 1406 | 39496 | 1189 | 1302 | 7416 | -0.5% | 0.16 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 115.6 | 34.6 | 89.4 | 25.7 | 30.5 | 46.6 | -22.7% | 0.86 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1846 | 565.4 | 1506 | 364.2 | 475.6 | 500.5 | -18.4% | 0.71 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 203.6 | 44.8 | 234.3 | 26.6 | 36.8 | 42.1 | +15.1% | 0.83 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 160.1 | 31.3 | 164.2 | 8.87 | 23.0 | 21.2 | +2.6% | 0.18 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 3031 | 719.2 | 3501 | 666.2 | 693.2 | 1024 | +15.5% | 0.68 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 379.2 | 76.2 | 342.4 | 25.6 | 56.8 | 208.7 | -9.7% | 0.65 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 8658 | 4705 | 10453 | 6486 | 5666 | 15420 | +20.7% | 0.32 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 2794516 | 27288 | 2834172 | 38525 | 33382 | 94111 | +1.4% | 1.19 | BELOW FLOOR |
| P vs M | write_p99_us | 3043616 | 54346 | 3048386 | 52352 | 53358 | 105548 | +0.2% | 0.09 | BELOW FLOOR |
| P vs M | commits_per_s | 0.36 | 0.00 | 0.35 | 0.00 | 0.00 | 0.01 | -1.3% | 1.08 | BELOW FLOOR |
| P vs M | pss_mib | 488.5 | 1.26 | 486.7 | 2.93 | 2.26 | 6.73 | -0.4% | 0.81 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 53326 | 12596 | 44815 | 19422 | 16369 | 189055 | -16.0% | 0.52 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10³ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 117.5 | 4.99 | 597.9 | 22.6 | 16.4 | 34.8 | +408.9% | 29.3 | **N better** |
| N vs P+ | q1_p99_us | 404.0 | 156.7 | 2754 | 1699 | 1206 | 242.2 | +581.6% | 1.95 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 133.9 | 3.54 | 471.7 | 30.8 | 21.9 | 2.38 | +252.3% | 15.4 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2276 | 34.1 | 3780 | 108.2 | 80.2 | 52.7 | +66.1% | 18.8 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 165.3 | 6.29 | 709.0 | 41.8 | 29.9 | 240.0 | +328.8% | 18.2 | **N better** |
| N vs P+ | c2_q1_p99_us | 662.8 | 238.6 | 5368 | 863.5 | 633.4 | 1504 | +709.8% | 7.43 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 184.7 | 19.1 | 634.4 | 43.6 | 33.7 | 116.0 | +243.4% | 13.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 236.8 | 47.0 | 1896 | 121.5 | 92.1 | 130.8 | +700.6% | 18.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 833.5 | 110.5 | 8311 | 1537 | 1090 | 5017 | +897.1% | 6.86 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 269.0 | 60.9 | 1569 | 73.5 | 67.5 | 88.3 | +483.3% | 19.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 855.8 | 103.4 | 9106 | 2017 | 1428 | 4860 | +964.1% | 5.78 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 378.1 | 31.6 | 813.6 | 46.5 | 39.8 | 29.4 | +115.2% | 10.9 | **N better** |
| N vs P+ | write_p99_us | 810.7 | 113.3 | 2806 | 195.1 | 159.5 | 174.1 | +246.1% | 12.5 | **N better** |
| N vs P+ | commits_per_s | 2345 | 135.5 | 1013 | 87.1 | 113.9 | 98.5 | -56.8% | 11.7 | **N better** |
| N vs P+ | pss_mib | 13.1 | 0.63 | 45.4 | 0.01 | 0.45 | 0.17 | +245.8% | 72.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 9056 | 296.0 | -3033 | 132.5 | 229.3 | 6571 | -133.5% | 52.7 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 117.5 | 4.99 | 120.6 | 4.17 | 4.60 | 19.6 | +2.6% | 0.67 | BELOW FLOOR |
| N vs P | q1_p99_us | 404.0 | 156.7 | 426.6 | 110.2 | 135.5 | 221.0 | +5.6% | 0.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | q2_p50_us | 133.9 | 3.54 | 194.7 | 10.2 | 7.63 | 21.8 | +45.4% | 7.97 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2276 | 34.1 | 152.9 | 5.81 | 24.4 | 55.3 | -93.3% | 86.9 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 165.3 | 6.29 | 141.9 | 16.5 | 12.5 | 22.4 | -14.1% | 1.88 | no difference |
| N vs P | c2_q1_p99_us | 662.8 | 238.6 | 1649 | 551.1 | 424.6 | 308.1 | +148.8% | 2.32 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 184.7 | 19.1 | 246.2 | 30.6 | 25.5 | 52.7 | +33.3% | 2.41 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 236.8 | 47.0 | 234.7 | 49.0 | 48.0 | 109.0 | -0.9% | 0.04 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 833.5 | 110.5 | 2284 | 252.0 | 194.6 | 575.6 | +174.0% | 7.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 269.0 | 60.9 | 298.5 | 36.0 | 50.0 | 98.4 | +11.0% | 0.59 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 855.8 | 103.4 | 2620 | 493.0 | 356.2 | 1558 | +206.2% | 4.95 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 378.1 | 31.6 | 14893 | 390.8 | 277.3 | 1141 | +3839.4% | 52.4 | **N better** |
| N vs P | write_p99_us | 810.7 | 113.3 | 24666 | 5247 | 3711 | 2205 | +2942.6% | 6.43 | **N better** |
| N vs P | commits_per_s | 2345 | 135.5 | 62.0 | 3.26 | 95.8 | 98.2 | -97.4% | 23.8 | **N better** |
| N vs P | pss_mib | 13.1 | 0.63 | 45.3 | 0.04 | 0.45 | 0.18 | +245.6% | 71.9 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 9056 | 296.0 | -2788 | 138.3 | 231.0 | 6508 | -130.8% | 51.3 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 597.9 | 22.6 | 120.6 | 4.17 | 16.3 | 38.1 | -79.8% | 29.4 | **P better** |
| P+ vs P | q1_p99_us | 2754 | 1699 | 426.6 | 110.2 | 1204 | 105.4 | -84.5% | 1.93 | no difference |
| P+ vs P | q2_p50_us | 471.7 | 30.8 | 194.7 | 10.2 | 22.9 | 21.7 | -58.7% | 12.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 222.7 | 16.0 | 227.6 | 13.1 | 14.6 | 59.5 | +2.2% | 0.34 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1510 | 46.9 | 227.0 | 17.4 | 35.4 | 366.4 | -85.0% | 36.3 | **P better** |
| P+ vs P | q5_p50_us | 3780 | 108.2 | 152.9 | 5.81 | 76.7 | 62.5 | -96.0% | 47.3 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 709.0 | 41.8 | 141.9 | 16.5 | 31.8 | 239.1 | -80.0% | 17.8 | **P better** |
| P+ vs P | c2_q1_p99_us | 5368 | 863.5 | 1649 | 551.1 | 724.3 | 1479 | -69.3% | 5.13 | **P better** |
| P+ vs P | c2_q2_p50_us | 634.4 | 43.6 | 246.2 | 30.6 | 37.7 | 104.5 | -61.2% | 10.3 | **P better** |
| P+ vs P | c4_q1_p50_us | 1896 | 121.5 | 234.7 | 49.0 | 92.7 | 156.3 | -87.6% | 17.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8311 | 1537 | 2284 | 252.0 | 1102 | 4998 | -72.5% | 5.47 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1569 | 73.5 | 298.5 | 36.0 | 57.9 | 85.7 | -81.0% | 22.0 | **P better** |
| P+ vs P | c4_q2_p99_us | 9106 | 2017 | 2620 | 493.0 | 1468 | 4924 | -71.2% | 4.42 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 813.6 | 46.5 | 14893 | 390.8 | 278.3 | 1141 | +1730.5% | 50.6 | **P+ better** |
| P+ vs P | write_p99_us | 2806 | 195.1 | 24666 | 5247 | 3713 | 2209 | +779.1% | 5.89 | **P+ better** |
| P+ vs P | commits_per_s | 1013 | 87.1 | 62.0 | 3.26 | 61.6 | 8.22 | -93.9% | 15.4 | **P+ better** |
| P+ vs P | pss_mib | 45.4 | 0.01 | 45.3 | 0.04 | 0.03 | 0.04 | -0.1% | 1.05 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -3033 | 132.5 | -2788 | 138.3 | 135.4 | 3236 | -8.1% | 1.81 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 120.6 | 4.17 | 120.9 | 6.48 | 5.45 | 17.9 | +0.3% | 0.06 | BELOW FLOOR |
| P vs M | q1_p99_us | 426.6 | 110.2 | 606.8 | 305.3 | 229.5 | 98.3 | +42.2% | 0.79 | no difference |
| P vs M | q2_p50_us | 194.7 | 10.2 | 205.8 | 13.8 | 12.1 | 28.1 | +5.7% | 0.92 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 227.6 | 13.1 | 227.0 | 9.34 | 11.4 | 40.6 | -0.3% | 0.05 | BELOW FLOOR |
| P vs M | q4_p50_us | 227.0 | 17.4 | 218.9 | 17.2 | 17.3 | 65.7 | -3.6% | 0.47 | BELOW FLOOR |
| P vs M | q5_p50_us | 152.9 | 5.81 | 156.6 | 10.0 | 8.19 | 45.9 | +2.4% | 0.45 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 141.9 | 16.5 | 55.8 | 8.91 | 13.2 | 13.5 | -60.7% | 6.51 | **M better** |
| P vs M | c2_q1_p99_us | 1649 | 551.1 | 1201 | 324.0 | 452.0 | 155.4 | -27.2% | 0.99 | no difference |
| P vs M | c2_q2_p50_us | 246.2 | 30.6 | 140.1 | 22.5 | 26.9 | 23.9 | -43.1% | 3.95 | **M better** |
| P vs M | c4_q1_p50_us | 234.7 | 49.0 | 197.0 | 49.4 | 49.2 | 104.7 | -16.1% | 0.77 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2284 | 252.0 | 1841 | 277.9 | 265.3 | 1376 | -19.4% | 1.67 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 298.5 | 36.0 | 280.7 | 33.8 | 34.9 | 90.0 | -6.0% | 0.51 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2620 | 493.0 | 2539 | 827.5 | 681.1 | 1503 | -3.1% | 0.12 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 14893 | 390.8 | 14795 | 248.3 | 327.4 | 1160 | -0.7% | 0.30 | BELOW FLOOR |
| P vs M | write_p99_us | 24666 | 5247 | 22419 | 2283 | 4046 | 2246 | -9.1% | 0.56 | no difference |
| P vs M | commits_per_s | 62.0 | 3.26 | 63.3 | 1.34 | 2.50 | 2.18 | +2.1% | 0.51 | BELOW FLOOR |
| P vs M | pss_mib | 45.3 | 0.04 | 45.1 | 0.07 | 0.06 | 0.04 | -0.4% | 3.37 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2788 | 138.3 | -2819 | 194.8 | 168.9 | 7836 | +1.1% | 0.18 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10³ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 120.7 | 1.92 | 556.1 | 45.1 | 31.9 | 76.1 | +360.7% | 13.6 | **N better** |
| N vs P+ | q1_p99_us | 287.5 | 32.8 | 1427 | 342.2 | 243.1 | 3473 | +396.5% | 4.69 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 140.0 | 8.73 | 427.1 | 40.7 | 29.4 | 82.1 | +205.1% | 9.75 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2407 | 48.4 | 3777 | 142.8 | 106.6 | 33.6 | +56.9% | 12.8 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 154.4 | 18.8 | 597.9 | 22.6 | 20.7 | 158.9 | +287.3% | 21.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 480.0 | 130.1 | 6097 | 726.1 | 521.6 | 77.9 | +1170.3% | 10.8 | **N better** |
| N vs P+ | c2_q2_p50_us | 173.8 | 31.2 | 528.4 | 35.5 | 33.4 | 193.2 | +204.0% | 10.6 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q1_p50_us | 162.9 | 20.8 | 1810 | 98.7 | 71.3 | 38.1 | +1011.3% | 23.1 | **N better** |
| N vs P+ | c4_q1_p99_us | 657.7 | 101.5 | 13359 | 7023 | 4967 | 6274 | +1931.3% | 2.56 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 200.8 | 36.9 | 1535 | 38.8 | 37.8 | 122.4 | +664.4% | 35.3 | **N better** |
| N vs P+ | c4_q2_p99_us | 877.0 | 221.3 | 8210 | 3008 | 2133 | 8212 | +836.2% | 3.44 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 419.2 | 23.8 | 778.4 | 118.0 | 85.1 | 106.1 | +85.7% | 4.22 | **N better** |
| N vs P+ | write_p99_us | 794.1 | 73.5 | 2650 | 183.3 | 139.6 | 151.5 | +233.7% | 13.3 | **N better** |
| N vs P+ | commits_per_s | 2201 | 127.8 | 1046 | 125.1 | 126.5 | 290.1 | -52.5% | 9.13 | **N better** |
| N vs P+ | pss_mib | 13.1 | 0.45 | 45.6 | 0.02 | 0.32 | 0.24 | +247.0% | 102.1 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 8669 | 669.8 | -2523 | 140.1 | 483.9 | 3468 | -129.1% | 23.1 | **P+ better** |
| N vs P | q1_p50_us | 120.7 | 1.92 | 124.9 | 4.65 | 3.56 | 37.3 | +3.4% | 1.16 | BELOW FLOOR |
| N vs P | q1_p99_us | 287.5 | 32.8 | 553.1 | 190.6 | 136.8 | 265.5 | +92.4% | 1.94 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 140.0 | 8.73 | 212.6 | 22.5 | 17.1 | 40.1 | +51.9% | 4.26 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2407 | 48.4 | 159.9 | 10.6 | 35.1 | 32.5 | -93.4% | 64.1 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 154.4 | 18.8 | 120.6 | 7.18 | 14.2 | 79.2 | -21.9% | 2.38 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 480.0 | 130.1 | 1731 | 273.6 | 214.2 | 1143 | +260.7% | 5.84 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 173.8 | 31.2 | 212.5 | 15.9 | 24.8 | 142.9 | +22.3% | 1.56 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p50_us | 162.9 | 20.8 | 199.0 | 52.9 | 40.2 | 60.4 | +22.2% | 0.90 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 657.7 | 101.5 | 2018 | 401.8 | 293.0 | 581.7 | +206.8% | 4.64 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 200.8 | 36.9 | 301.7 | 35.1 | 36.0 | 110.4 | +50.3% | 2.80 | BELOW FLOOR |
| N vs P | c4_q2_p99_us | 877.0 | 221.3 | 1993 | 332.6 | 282.5 | 704.0 | +127.3% | 3.95 | **N better** |
| N vs P | write_p50_us | 419.2 | 23.8 | 15166 | 605.4 | 428.4 | 361.4 | +3518.0% | 34.4 | **N better** |
| N vs P | write_p99_us | 794.1 | 73.5 | 22340 | 3728 | 2637 | 5268 | +2713.3% | 8.17 | **N better** |
| N vs P | commits_per_s | 2201 | 127.8 | 63.0 | 4.93 | 90.5 | 290.3 | -97.1% | 23.6 | **N better** |
| N vs P | pss_mib | 13.1 | 0.45 | 45.3 | 0.01 | 0.32 | 0.23 | +244.4% | 101.1 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 8669 | 669.8 | -2793 | 99.0 | 478.8 | 4055 | -132.2% | 23.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q1_p50_us | 556.1 | 45.1 | 124.9 | 4.65 | 32.1 | 84.4 | -77.5% | 13.4 | **P better** |
| P+ vs P | q1_p99_us | 1427 | 342.2 | 553.1 | 190.6 | 277.0 | 3483 | -61.2% | 3.16 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 427.1 | 40.7 | 212.6 | 22.5 | 32.9 | 89.6 | -50.2% | 6.52 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 230.3 | 16.3 | 238.8 | 13.1 | 14.8 | 90.6 | +3.7% | 0.58 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1470 | 47.2 | 228.9 | 6.87 | 33.7 | 160.3 | -84.4% | 36.8 | **P better** |
| P+ vs P | q5_p50_us | 3777 | 142.8 | 159.9 | 10.6 | 101.3 | 37.2 | -95.8% | 35.7 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 597.9 | 22.6 | 120.6 | 7.18 | 16.7 | 154.1 | -79.8% | 28.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 6097 | 726.1 | 1731 | 273.6 | 548.6 | 1142 | -71.6% | 7.96 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 528.4 | 35.5 | 212.5 | 15.9 | 27.5 | 225.3 | -59.8% | 11.5 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p50_us | 1810 | 98.7 | 199.0 | 52.9 | 79.2 | 71.3 | -89.0% | 20.3 | **P better** |
| P+ vs P | c4_q1_p99_us | 13359 | 7023 | 2018 | 401.8 | 4974 | 6301 | -84.9% | 2.28 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1535 | 38.8 | 301.7 | 35.1 | 37.0 | 162.5 | -80.3% | 33.3 | **P better** |
| P+ vs P | c4_q2_p99_us | 8210 | 3008 | 1993 | 332.6 | 2140 | 8240 | -75.7% | 2.91 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 778.4 | 118.0 | 15166 | 605.4 | 436.1 | 372.9 | +1848.5% | 33.0 | **P+ better** |
| P+ vs P | write_p99_us | 2650 | 183.3 | 22340 | 3728 | 2639 | 5266 | +743.1% | 7.46 | **P+ better** |
| P+ vs P | commits_per_s | 1046 | 125.1 | 63.0 | 4.93 | 88.6 | 12.9 | -94.0% | 11.1 | **P+ better** |
| P+ vs P | pss_mib | 45.6 | 0.02 | 45.3 | 0.01 | 0.02 | 0.08 | -0.8% | 18.1 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2523 | 140.1 | -2793 | 99.0 | 121.3 | 2297 | +10.7% | 2.23 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q1_p50_us | 124.9 | 4.65 | 119.1 | 5.68 | 5.19 | 40.7 | -4.6% | 1.11 | BELOW FLOOR |
| P vs M | q1_p99_us | 553.1 | 190.6 | 597.1 | 272.2 | 235.0 | 266.6 | +8.0% | 0.19 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 212.6 | 22.5 | 200.1 | 10.6 | 17.6 | 41.5 | -5.9% | 0.71 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 238.8 | 13.1 | 243.9 | 26.0 | 20.6 | 75.1 | +2.1% | 0.25 | BELOW FLOOR |
| P vs M | q4_p50_us | 228.9 | 6.87 | 220.9 | 7.47 | 7.18 | 37.6 | -3.5% | 1.12 | BELOW FLOOR |
| P vs M | q5_p50_us | 159.9 | 10.6 | 148.2 | 13.3 | 12.1 | 31.2 | -7.4% | 0.98 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 120.6 | 7.18 | 136.1 | 6.36 | 6.78 | 49.2 | +12.9% | 2.29 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1731 | 273.6 | 2187 | 163.9 | 225.5 | 1751 | +26.3% | 2.02 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 212.5 | 15.9 | 247.3 | 15.4 | 15.6 | 130.5 | +16.4% | 2.23 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 199.0 | 52.9 | 201.4 | 42.9 | 48.2 | 65.7 | +1.2% | 0.05 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2018 | 401.8 | 2424 | 309.6 | 358.7 | 680.6 | +20.1% | 1.13 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 301.7 | 35.1 | 298.4 | 36.4 | 35.8 | 109.1 | -1.1% | 0.09 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 1993 | 332.6 | 2735 | 495.0 | 421.7 | 1243 | +37.2% | 1.76 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 15166 | 605.4 | 15049 | 305.4 | 479.4 | 465.6 | -0.8% | 0.24 | BELOW FLOOR |
| P vs M | write_p99_us | 22340 | 3728 | 20273 | 2012 | 2996 | 5966 | -9.3% | 0.69 | BELOW FLOOR |
| P vs M | commits_per_s | 63.0 | 4.93 | 64.1 | 2.36 | 3.86 | 11.9 | +1.7% | 0.28 | BELOW FLOOR |
| P vs M | pss_mib | 45.3 | 0.01 | 45.3 | 0.04 | 0.03 | 0.04 | -0.0% | 0.05 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -2793 | 99.0 | -2746 | 123.5 | 111.9 | 2888 | -1.7% | 0.43 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10³ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 120.5 | 1.89 | 554.2 | 40.2 | 28.4 | 14.3 | +359.9% | 15.3 | **N better** |
| N vs P+ | q1_p99_us | 303.2 | 77.7 | 1499 | 485.6 | 347.8 | 142.0 | +394.5% | 3.44 | **N better** |
| N vs P+ | q2_p50_us | 140.0 | 6.14 | 441.1 | 30.5 | 22.0 | 11.5 | +215.1% | 13.7 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2373 | 51.9 | 3743 | 74.4 | 64.2 | 186.8 | +57.7% | 21.3 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 160.9 | 9.27 | 634.4 | 44.9 | 32.4 | 45.3 | +294.3% | 14.6 | **N better** |
| N vs P+ | c2_q1_p99_us | 405.1 | 48.9 | 5590 | 2894 | 2047 | 287.2 | +1279.9% | 2.53 | no difference |
| N vs P+ | c2_q2_p50_us | 187.6 | 35.7 | 519.4 | 23.3 | 30.1 | 87.5 | +176.9% | 11.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 221.2 | 28.9 | 1614 | 61.3 | 48.0 | 82.2 | +629.6% | 29.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 780.4 | 125.5 | 11348 | 4681 | 3311 | 3668 | +1354.2% | 3.19 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 240.8 | 20.0 | 1364 | 81.7 | 59.5 | 67.3 | +466.5% | 18.9 | **N better** |
| N vs P+ | c4_q2_p99_us | 948.8 | 274.8 | 8983 | 5279 | 3738 | 11351 | +846.7% | 2.15 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 393.2 | 16.5 | 794.9 | 73.5 | 53.2 | 49.1 | +102.1% | 7.54 | **N better** |
| N vs P+ | write_p99_us | 709.9 | 46.6 | 2589 | 154.3 | 114.0 | 110.2 | +264.7% | 16.5 | **N better** |
| N vs P+ | commits_per_s | 2342 | 141.4 | 1075 | 44.2 | 104.8 | 127.3 | -54.1% | 12.1 | **N better** |
| N vs P+ | pss_mib | 13.3 | 0.30 | 45.2 | 0.02 | 0.21 | 0.11 | +240.5% | 150.8 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 8882 | 156.2 | -2725 | 123.0 | 140.6 | 5536 | -130.7% | 82.6 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 120.5 | 1.89 | 120.8 | 7.21 | 5.27 | 3.91 | +0.2% | 0.05 | BELOW FLOOR |
| N vs P | q1_p99_us | 303.2 | 77.7 | 424.8 | 73.5 | 75.6 | 72.2 | +40.1% | 1.61 | no difference |
| N vs P | q2_p50_us | 140.0 | 6.14 | 206.2 | 15.7 | 11.9 | 29.4 | +47.3% | 5.55 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2373 | 51.9 | 137.3 | 10.4 | 37.4 | 185.1 | -94.2% | 59.7 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 160.9 | 9.27 | 125.0 | 15.8 | 12.9 | 49.2 | -22.3% | 2.77 | BELOW FLOOR |
| N vs P | c2_q1_p99_us | 405.1 | 48.9 | 1262 | 574.6 | 407.8 | 1102 | +211.6% | 2.10 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 187.6 | 35.7 | 221.0 | 29.5 | 32.8 | 85.0 | +17.8% | 1.02 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 221.2 | 28.9 | 200.5 | 38.0 | 33.8 | 69.2 | -9.4% | 0.61 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 780.4 | 125.5 | 2587 | 287.4 | 221.7 | 811.0 | +231.5% | 8.15 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 240.8 | 20.0 | 330.9 | 41.7 | 32.7 | 54.8 | +37.4% | 2.76 | no difference |
| N vs P | c4_q2_p99_us | 948.8 | 274.8 | 2582 | 264.7 | 269.8 | 406.5 | +172.1% | 6.05 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 393.2 | 16.5 | 14840 | 154.3 | 109.7 | 43.6 | +3673.6% | 131.7 | **N better** |
| N vs P | write_p99_us | 709.9 | 46.6 | 20952 | 2963 | 2096 | 1438 | +2851.4% | 9.66 | **N better** |
| N vs P | commits_per_s | 2342 | 141.4 | 64.8 | 2.22 | 100.0 | 126.8 | -97.2% | 22.8 | **N better** |
| N vs P | pss_mib | 13.3 | 0.30 | 45.4 | 0.02 | 0.21 | 0.11 | +242.0% | 151.5 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 8882 | 156.2 | -2939 | 166.2 | 161.3 | 5571 | -133.1% | 73.3 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 554.2 | 40.2 | 120.8 | 7.21 | 28.8 | 14.8 | -78.2% | 15.0 | **P better** |
| P+ vs P | q1_p99_us | 1499 | 485.6 | 424.8 | 73.5 | 347.3 | 126.0 | -71.7% | 3.09 | **P better** |
| P+ vs P | q2_p50_us | 441.1 | 30.5 | 206.2 | 15.7 | 24.3 | 31.2 | -53.3% | 9.68 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 234.5 | 18.7 | 233.4 | 21.6 | 20.2 | 96.4 | -0.5% | 0.06 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1521 | 113.5 | 239.9 | 21.8 | 81.7 | 46.8 | -84.2% | 15.7 | **P better** |
| P+ vs P | q5_p50_us | 3743 | 74.4 | 137.3 | 10.4 | 53.2 | 58.9 | -96.3% | 67.8 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 634.4 | 44.9 | 125.0 | 15.8 | 33.6 | 41.8 | -80.3% | 15.1 | **P better** |
| P+ vs P | c2_q1_p99_us | 5590 | 2894 | 1262 | 574.6 | 2086 | 1139 | -77.4% | 2.07 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 519.4 | 23.3 | 221.0 | 29.5 | 26.6 | 26.2 | -57.5% | 11.2 | **P better** |
| P+ vs P | c4_q1_p50_us | 1614 | 61.3 | 200.5 | 38.0 | 51.0 | 107.4 | -87.6% | 27.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 11348 | 4681 | 2587 | 287.4 | 3316 | 3756 | -77.2% | 2.64 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1364 | 81.7 | 330.9 | 41.7 | 64.9 | 56.9 | -75.7% | 15.9 | **P better** |
| P+ vs P | c4_q2_p99_us | 8983 | 5279 | 2582 | 264.7 | 3738 | 11344 | -71.3% | 1.71 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 794.9 | 73.5 | 14840 | 154.3 | 120.8 | 63.6 | +1766.8% | 116.2 | **P+ better** |
| P+ vs P | write_p99_us | 2589 | 154.3 | 20952 | 2963 | 2098 | 1442 | +709.3% | 8.75 | **P+ better** |
| P+ vs P | commits_per_s | 1075 | 44.2 | 64.8 | 2.22 | 31.3 | 12.2 | -94.0% | 32.3 | **P+ better** |
| P+ vs P | pss_mib | 45.2 | 0.02 | 45.4 | 0.02 | 0.02 | 0.00 | +0.4% | 9.53 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2725 | 123.0 | -2939 | 166.2 | 146.2 | 2855 | +7.8% | 1.46 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 120.8 | 7.21 | 118.4 | 4.37 | 5.96 | 4.68 | -2.0% | 0.41 | BELOW FLOOR |
| P vs M | q1_p99_us | 424.8 | 73.5 | 446.2 | 129.6 | 105.3 | 39.2 | +5.0% | 0.20 | BELOW FLOOR |
| P vs M | q2_p50_us | 206.2 | 15.7 | 193.8 | 5.59 | 11.8 | 29.5 | -6.0% | 1.05 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 233.4 | 21.6 | 229.7 | 31.2 | 26.8 | 117.2 | -1.6% | 0.14 | BELOW FLOOR |
| P vs M | q4_p50_us | 239.9 | 21.8 | 225.9 | 10.5 | 17.1 | 35.7 | -5.8% | 0.82 | BELOW FLOOR |
| P vs M | q5_p50_us | 137.3 | 10.4 | 139.4 | 11.2 | 10.8 | 37.9 | +1.6% | 0.20 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 125.0 | 15.8 | 133.6 | 9.87 | 13.2 | 35.8 | +6.9% | 0.66 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1262 | 574.6 | 1611 | 712.1 | 647.0 | 1296 | +27.7% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 221.0 | 29.5 | 243.1 | 11.0 | 22.3 | 19.2 | +10.0% | 0.99 | no difference |
| P vs M | c4_q1_p50_us | 200.5 | 38.0 | 195.3 | 17.4 | 29.5 | 76.8 | -2.6% | 0.18 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2587 | 287.4 | 2452 | 274.1 | 280.8 | 865.1 | -5.2% | 0.48 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 330.9 | 41.7 | 292.9 | 27.2 | 35.2 | 50.5 | -11.5% | 1.08 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2582 | 264.7 | 2746 | 397.9 | 337.9 | 155.8 | +6.4% | 0.49 | no difference |
| P vs M | write_p50_us | 14840 | 154.3 | 14572 | 213.5 | 186.2 | 320.8 | -1.8% | 1.44 | BELOW FLOOR |
| P vs M | write_p99_us | 20952 | 2963 | 18716 | 570.2 | 2134 | 2320 | -10.7% | 1.05 | BELOW FLOOR |
| P vs M | commits_per_s | 64.8 | 2.22 | 66.5 | 0.75 | 1.66 | 4.29 | +2.6% | 1.02 | BELOW FLOOR |
| P vs M | pss_mib | 45.4 | 0.02 | 45.4 | 0.06 | 0.04 | 0.01 | +0.1% | 1.34 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2939 | 166.2 | -2746 | 133.1 | 150.6 | 2721 | -6.6% | 1.28 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10³ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 123.2 | 7.80 | 547.2 | 23.2 | 17.3 | 15.8 | +344.1% | 24.5 | **N better** |
| N vs P+ | q1_p99_us | 330.2 | 37.4 | 1142 | 237.4 | 169.9 | 212.8 | +245.8% | 4.78 | **N better** |
| N vs P+ | q2_p50_us | 133.5 | 2.90 | 415.4 | 11.6 | 8.44 | 44.8 | +211.2% | 33.4 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2428 | 47.6 | 3816 | 65.6 | 57.3 | 270.7 | +57.2% | 24.2 | **N better** |
| N vs P+ | q6_p50_us | 3351 | 100.0 | 3907 | 83.3 | 92.0 | 967.0 | +16.6% | 6.04 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 96.5 | 32.4 | 606.6 | 25.4 | 29.1 | 65.6 | +528.8% | 17.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 436.2 | 212.7 | 5865 | 793.2 | 580.7 | 502.3 | +1244.7% | 9.35 | **N better** |
| N vs P+ | c2_q2_p50_us | 98.4 | 26.2 | 528.2 | 44.0 | 36.2 | 105.1 | +436.9% | 11.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 140.5 | 11.4 | 1687 | 124.9 | 88.7 | 152.0 | +1100.7% | 17.4 | **N better** |
| N vs P+ | c4_q1_p99_us | 620.1 | 100.4 | 507307 | 497402 | 351716 | 7202 | +81708.7% | 1.44 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 167.7 | 14.8 | 1440 | 117.8 | 84.0 | 137.7 | +758.7% | 15.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 715.1 | 130.1 | 8493 | 2347 | 1662 | 1347 | +1087.6% | 4.68 | **N better** |
| N vs P+ | write_p50_us | 415.7 | 38.9 | 773.5 | 44.0 | 41.5 | 56.0 | +86.1% | 8.62 | **N better** |
| N vs P+ | write_p99_us | 773.8 | 122.5 | 2689 | 96.0 | 110.1 | 697.8 | +247.5% | 17.4 | **N better** |
| N vs P+ | commits_per_s | 2221 | 307.6 | 1111 | 58.5 | 221.4 | 233.9 | -50.0% | 5.01 | **N better** |
| N vs P+ | pss_mib | 13.5 | 0.49 | 45.1 | 0.03 | 0.35 | 0.15 | +235.0% | 91.2 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 9248 | 306.8 | -2783 | 105.7 | 229.5 | 5401 | -130.1% | 52.4 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 123.2 | 7.80 | 118.9 | 10.2 | 9.08 | 13.0 | -3.5% | 0.48 | BELOW FLOOR |
| N vs P | q1_p99_us | 330.2 | 37.4 | 448.7 | 130.8 | 96.2 | 107.0 | +35.9% | 1.23 | no difference |
| N vs P | q2_p50_us | 133.5 | 2.90 | 209.8 | 13.6 | 9.82 | 15.8 | +57.2% | 7.78 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2428 | 47.6 | 143.7 | 7.51 | 34.1 | 237.9 | -94.1% | 67.0 | **P better** |
| N vs P | q6_p50_us | 3351 | 100.0 | 533.5 | 44.3 | 77.3 | 934.0 | -84.1% | 36.4 | **P better** |
| N vs P | c2_q1_p50_us | 96.5 | 32.4 | 50.3 | 2.32 | 23.0 | 60.6 | -47.8% | 2.01 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 436.2 | 212.7 | 1067 | 63.2 | 156.9 | 1214 | +144.7% | 4.02 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 98.4 | 26.2 | 112.4 | 16.4 | 21.9 | 96.8 | +14.3% | 0.64 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 140.5 | 11.4 | 162.8 | 39.4 | 29.0 | 28.2 | +15.9% | 0.77 | BELOW FLOOR |
| N vs P | c4_q1_p99_us | 620.1 | 100.4 | 2176 | 314.2 | 233.3 | 204.0 | +251.0% | 6.67 | **N better** |
| N vs P | c4_q2_p50_us | 167.7 | 14.8 | 257.8 | 4.68 | 10.9 | 78.0 | +53.7% | 8.23 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 715.1 | 130.1 | 2207 | 242.4 | 194.5 | 254.5 | +208.7% | 7.67 | **N better** |
| N vs P | write_p50_us | 415.7 | 38.9 | 15018 | 410.8 | 291.8 | 349.8 | +3512.6% | 50.0 | **N better** |
| N vs P | write_p99_us | 773.8 | 122.5 | 22733 | 3840 | 2717 | 765.2 | +2837.7% | 8.08 | **N better** |
| N vs P | commits_per_s | 2221 | 307.6 | 62.4 | 3.63 | 217.5 | 230.0 | -97.2% | 9.92 | **N better** |
| N vs P | pss_mib | 13.5 | 0.49 | 45.2 | 0.01 | 0.35 | 0.15 | +235.5% | 91.5 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 9248 | 306.8 | -2827 | 129.1 | 235.4 | 5426 | -130.6% | 51.3 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 547.2 | 23.2 | 118.9 | 10.2 | 17.9 | 14.0 | -78.3% | 23.9 | **P better** |
| P+ vs P | q1_p99_us | 1142 | 237.4 | 448.7 | 130.8 | 191.6 | 237.0 | -60.7% | 3.62 | **P better** |
| P+ vs P | q2_p50_us | 415.4 | 11.6 | 209.8 | 13.6 | 12.6 | 46.8 | -49.5% | 16.3 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 239.6 | 18.6 | 251.1 | 44.4 | 34.0 | 22.5 | +4.8% | 0.34 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1518 | 75.5 | 230.4 | 14.4 | 54.4 | 13.8 | -84.8% | 23.7 | **P better** |
| P+ vs P | q5_p50_us | 3816 | 65.6 | 143.7 | 7.51 | 46.7 | 133.8 | -96.2% | 78.7 | **P better** |
| P+ vs P | q6_p50_us | 3907 | 83.3 | 533.5 | 44.3 | 66.7 | 253.5 | -86.3% | 50.6 | **P better** |
| P+ vs P | c2_q1_p50_us | 606.6 | 25.4 | 50.3 | 2.32 | 18.0 | 26.0 | -91.7% | 30.9 | **P better** |
| P+ vs P | c2_q1_p99_us | 5865 | 793.2 | 1067 | 63.2 | 562.7 | 1314 | -81.8% | 8.53 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 528.2 | 44.0 | 112.4 | 16.4 | 33.2 | 50.0 | -78.7% | 12.5 | **P better** |
| P+ vs P | c4_q1_p50_us | 1687 | 124.9 | 162.8 | 39.4 | 92.6 | 154.5 | -90.3% | 16.5 | **P better** |
| P+ vs P | c4_q1_p99_us | 507307 | 497402 | 2176 | 314.2 | 351716 | 7204 | -99.6% | 1.44 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1440 | 117.8 | 257.8 | 4.68 | 83.4 | 127.0 | -82.1% | 14.2 | **P better** |
| P+ vs P | c4_q2_p99_us | 8493 | 2347 | 2207 | 242.4 | 1668 | 1371 | -74.0% | 3.77 | **P better** |
| P+ vs P | write_p50_us | 773.5 | 44.0 | 15018 | 410.8 | 292.2 | 354.2 | +1841.6% | 48.8 | **P+ better** |
| P+ vs P | write_p99_us | 2689 | 96.0 | 22733 | 3840 | 2716 | 1036 | +745.4% | 7.38 | **P+ better** |
| P+ vs P | commits_per_s | 1111 | 58.5 | 62.4 | 3.63 | 41.4 | 42.7 | -94.4% | 25.3 | **P+ better** |
| P+ vs P | pss_mib | 45.1 | 0.03 | 45.2 | 0.01 | 0.02 | 0.02 | +0.1% | 2.81 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2783 | 105.7 | -2827 | 129.1 | 118.0 | 2646 | +1.6% | 0.37 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 118.9 | 10.2 | 116.9 | 4.33 | 7.84 | 8.68 | -1.7% | 0.25 | BELOW FLOOR |
| P vs M | q1_p99_us | 448.7 | 130.8 | 402.3 | 73.1 | 106.0 | 273.3 | -10.3% | 0.44 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q2_p50_us | 209.8 | 13.6 | 204.4 | 12.5 | 13.1 | 28.9 | -2.6% | 0.42 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 251.1 | 44.4 | 229.4 | 16.1 | 33.4 | 49.5 | -8.6% | 0.65 | BELOW FLOOR |
| P vs M | q4_p50_us | 230.4 | 14.4 | 217.9 | 23.8 | 19.7 | 62.9 | -5.4% | 0.64 | BELOW FLOOR |
| P vs M | q5_p50_us | 143.7 | 7.51 | 146.3 | 7.79 | 7.65 | 44.0 | +1.8% | 0.34 | BELOW FLOOR |
| P vs M | q6_p50_us | 533.5 | 44.3 | 516.4 | 44.5 | 44.4 | 28.9 | -3.2% | 0.39 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 50.3 | 2.32 | 117.8 | 21.1 | 15.0 | 31.3 | +134.1% | 4.50 | **P better** |
| P vs M | c2_q1_p99_us | 1067 | 63.2 | 1154 | 101.1 | 84.3 | 1330 | +8.1% | 1.02 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 112.4 | 16.4 | 209.7 | 25.4 | 21.4 | 24.9 | +86.5% | 4.54 | **P better** |
| P vs M | c4_q1_p50_us | 162.8 | 39.4 | 245.8 | 12.4 | 29.2 | 47.5 | +50.9% | 2.84 | no difference |
| P vs M | c4_q1_p99_us | 2176 | 314.2 | 2132 | 472.9 | 401.5 | 208.1 | -2.1% | 0.11 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 257.8 | 4.68 | 310.3 | 30.3 | 21.7 | 43.7 | +20.4% | 2.42 | no difference |
| P vs M | c4_q2_p99_us | 2207 | 242.4 | 2563 | 607.8 | 462.7 | 345.8 | +16.1% | 0.77 | no difference |
| P vs M | write_p50_us | 15018 | 410.8 | 14935 | 353.0 | 383.0 | 361.4 | -0.6% | 0.22 | BELOW FLOOR |
| P vs M | write_p99_us | 22733 | 3840 | 19255 | 819.2 | 2776 | 2644 | -15.3% | 1.25 | no difference |
| P vs M | commits_per_s | 62.4 | 3.63 | 65.3 | 1.74 | 2.84 | 4.10 | +4.6% | 1.01 | BELOW FLOOR |
| P vs M | pss_mib | 45.2 | 0.01 | 45.4 | 0.09 | 0.06 | 0.04 | +0.5% | 3.84 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -2827 | 129.1 | -2640 | 233.4 | 188.6 | 3016 | -6.6% | 0.99 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10³ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 116.9 | 2.19 | 544.7 | 11.0 | 7.91 | 4.07 | +365.9% | 54.1 | **N better** |
| N vs P+ | q1_p99_us | 311.2 | 34.4 | 1291 | 341.4 | 242.7 | 113.3 | +314.8% | 4.04 | **N better** |
| N vs P+ | q2_p50_us | 136.2 | 6.57 | 419.8 | 9.77 | 8.33 | 8.44 | +208.3% | 34.1 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 2365 | 34.1 | 3708 | 39.4 | 36.9 | 217.3 | +56.8% | 36.4 | **N better** |
| N vs P+ | q6_p50_us | 3284 | 51.5 | 3928 | 113.6 | 88.2 | 184.9 | +19.6% | 7.30 | **N better** |
| N vs P+ | c2_q1_p50_us | 71.3 | 4.82 | 622.9 | 25.0 | 18.0 | 83.6 | +773.4% | 30.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 244.0 | 63.0 | 5627 | 644.7 | 458.1 | 2498 | +2206.3% | 11.8 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 80.3 | 7.86 | 555.0 | 22.4 | 16.8 | 73.0 | +591.4% | 28.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 130.1 | 19.3 | 1703 | 83.7 | 60.7 | 98.8 | +1209.6% | 25.9 | **N better** |
| N vs P+ | c4_q1_p99_us | 500.5 | 95.6 | 9208 | 3328 | 2354 | 473.2 | +1739.6% | 3.70 | **N better** |
| N vs P+ | c4_q2_p50_us | 147.9 | 3.14 | 1427 | 46.7 | 33.1 | 482.0 | +865.3% | 38.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p99_us | 650.6 | 62.4 | 8154 | 3244 | 2294 | 13273 | +1153.3% | 3.27 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 408.5 | 37.3 | 772.9 | 26.8 | 32.5 | 50.0 | +89.2% | 11.2 | **N better** |
| N vs P+ | write_p99_us | 711.0 | 40.7 | 2576 | 87.3 | 68.1 | 565.3 | +262.3% | 27.4 | **N better** |
| N vs P+ | commits_per_s | 2229 | 90.1 | 1084 | 73.0 | 82.0 | 100.7 | -51.4% | 14.0 | **N better** |
| N vs P+ | pss_mib | 13.6 | 0.45 | 45.3 | 0.02 | 0.32 | 0.10 | +234.0% | 100.7 | **N better** |
| N vs P+ | pss_growth_bytes_per_key_read | 9098 | 143.9 | -2695 | 128.1 | 136.2 | 16030 | -129.6% | 86.6 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 116.9 | 2.19 | 118.6 | 4.32 | 3.42 | 8.42 | +1.4% | 0.49 | BELOW FLOOR |
| N vs P | q1_p99_us | 311.2 | 34.4 | 352.3 | 31.0 | 32.8 | 141.2 | +13.2% | 1.25 | BELOW FLOOR |
| N vs P | q2_p50_us | 136.2 | 6.57 | 208.6 | 13.0 | 10.3 | 28.7 | +53.2% | 7.02 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 2365 | 34.1 | 149.6 | 11.1 | 25.4 | 193.3 | -93.7% | 87.2 | **P better** |
| N vs P | q6_p50_us | 3284 | 51.5 | 676.0 | 53.1 | 52.3 | 185.6 | -79.4% | 49.9 | **P better** |
| N vs P | c2_q1_p50_us | 71.3 | 4.82 | 127.2 | 3.43 | 4.18 | 84.0 | +78.4% | 13.4 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 244.0 | 63.0 | 1189 | 165.8 | 125.4 | 496.4 | +387.2% | 7.53 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 80.3 | 7.86 | 226.5 | 30.2 | 22.1 | 83.7 | +182.1% | 6.62 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 130.1 | 19.3 | 244.5 | 21.8 | 20.6 | 17.2 | +88.0% | 5.55 | **N better** |
| N vs P | c4_q1_p99_us | 500.5 | 95.6 | 2318 | 346.7 | 254.3 | 1436 | +363.2% | 7.15 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 147.9 | 3.14 | 322.8 | 19.0 | 13.6 | 92.5 | +118.3% | 12.9 | **N better** |
| N vs P | c4_q2_p99_us | 650.6 | 62.4 | 2760 | 237.6 | 173.7 | 584.6 | +324.1% | 12.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 408.5 | 37.3 | 14931 | 347.3 | 247.0 | 703.5 | +3554.8% | 58.8 | **N better** |
| N vs P | write_p99_us | 711.0 | 40.7 | 20981 | 2890 | 2044 | 3468 | +2851.0% | 9.92 | **N better** |
| N vs P | commits_per_s | 2229 | 90.1 | 64.1 | 2.31 | 63.7 | 91.1 | -97.1% | 34.0 | **N better** |
| N vs P | pss_mib | 13.6 | 0.45 | 45.3 | 0.01 | 0.32 | 0.09 | +234.0% | 100.8 | **N better** |
| N vs P | pss_growth_bytes_per_key_read | 9098 | 143.9 | -3023 | 159.4 | 151.9 | 16051 | -133.2% | 79.8 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 544.7 | 11.0 | 118.6 | 4.32 | 8.33 | 8.97 | -78.2% | 51.1 | **P better** |
| P+ vs P | q1_p99_us | 1291 | 341.4 | 352.3 | 31.0 | 242.4 | 125.8 | -72.7% | 3.87 | **P better** |
| P+ vs P | q2_p50_us | 419.8 | 9.77 | 208.6 | 13.0 | 11.5 | 29.9 | -50.3% | 18.3 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 232.5 | 15.9 | 234.0 | 13.7 | 14.8 | 35.5 | +0.6% | 0.10 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 1479 | 31.9 | 226.9 | 18.4 | 26.1 | 45.4 | -84.7% | 48.1 | **P better** |
| P+ vs P | q5_p50_us | 3708 | 39.4 | 149.6 | 11.1 | 29.0 | 113.3 | -96.0% | 122.9 | **P better** |
| P+ vs P | q6_p50_us | 3928 | 113.6 | 676.0 | 53.1 | 88.7 | 56.9 | -82.8% | 36.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 622.9 | 25.0 | 127.2 | 3.43 | 17.8 | 54.0 | -79.6% | 27.8 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5627 | 644.7 | 1189 | 165.8 | 470.7 | 2474 | -78.9% | 9.43 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 555.0 | 22.4 | 226.5 | 30.2 | 26.6 | 47.0 | -59.2% | 12.3 | **P better** |
| P+ vs P | c4_q1_p50_us | 1703 | 83.7 | 244.5 | 21.8 | 61.2 | 97.3 | -85.6% | 23.8 | **P better** |
| P+ vs P | c4_q1_p99_us | 9208 | 3328 | 2318 | 346.7 | 2366 | 1503 | -74.8% | 2.91 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1427 | 46.7 | 322.8 | 19.0 | 35.7 | 489.0 | -77.4% | 31.0 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p99_us | 8154 | 3244 | 2760 | 237.6 | 2300 | 13274 | -66.2% | 2.35 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 772.9 | 26.8 | 14931 | 347.3 | 246.3 | 703.8 | +1831.8% | 57.5 | **P+ better** |
| P+ vs P | write_p99_us | 2576 | 87.3 | 20981 | 2890 | 2045 | 3513 | +714.4% | 9.00 | **P+ better** |
| P+ vs P | commits_per_s | 1084 | 73.0 | 64.1 | 2.31 | 51.7 | 43.3 | -94.1% | 19.7 | **P+ better** |
| P+ vs P | pss_mib | 45.3 | 0.02 | 45.3 | 0.01 | 0.02 | 0.10 | -0.0% | 0.25 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -2695 | 128.1 | -3023 | 159.4 | 144.6 | 2625 | +12.2% | 2.27 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 118.6 | 4.32 | 117.0 | 7.41 | 6.06 | 8.23 | -1.3% | 0.26 | BELOW FLOOR |
| P vs M | q1_p99_us | 352.3 | 31.0 | 344.8 | 23.4 | 27.5 | 112.1 | -2.1% | 0.27 | BELOW FLOOR |
| P vs M | q2_p50_us | 208.6 | 13.0 | 208.6 | 14.4 | 13.7 | 30.4 | +0.0% | 0.00 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 234.0 | 13.7 | 221.9 | 20.0 | 17.2 | 15.5 | -5.2% | 0.71 | BELOW FLOOR |
| P vs M | q4_p50_us | 226.9 | 18.4 | 224.0 | 16.6 | 17.5 | 43.0 | -1.3% | 0.17 | BELOW FLOOR |
| P vs M | q5_p50_us | 149.6 | 11.1 | 153.8 | 16.1 | 13.8 | 40.1 | +2.8% | 0.30 | BELOW FLOOR |
| P vs M | q6_p50_us | 676.0 | 53.1 | 603.8 | 51.0 | 52.0 | 68.4 | -10.7% | 1.39 | no difference |
| P vs M | c2_q1_p50_us | 127.2 | 3.43 | 126.6 | 6.70 | 5.32 | 47.4 | -0.5% | 0.12 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1189 | 165.8 | 1410 | 391.1 | 300.4 | 499.0 | +18.6% | 0.74 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 226.5 | 30.2 | 219.9 | 23.0 | 26.9 | 52.6 | -2.9% | 0.24 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 244.5 | 21.8 | 249.0 | 14.4 | 18.5 | 5.52 | +1.8% | 0.24 | BELOW FLOOR |
| P vs M | c4_q1_p99_us | 2318 | 346.7 | 2377 | 148.3 | 266.6 | 1431 | +2.5% | 0.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 322.8 | 19.0 | 332.2 | 45.5 | 34.9 | 95.4 | +2.9% | 0.27 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 2760 | 237.6 | 2589 | 236.0 | 236.8 | 1223 | -6.2% | 0.72 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 14931 | 347.3 | 14608 | 136.0 | 263.8 | 710.7 | -2.2% | 1.22 | BELOW FLOOR |
| P vs M | write_p99_us | 20981 | 2890 | 20493 | 2500 | 2702 | 3469 | -2.3% | 0.18 | BELOW FLOOR |
| P vs M | commits_per_s | 64.1 | 2.31 | 65.4 | 2.08 | 2.19 | 3.62 | +2.1% | 0.60 | BELOW FLOOR |
| P vs M | pss_mib | 45.3 | 0.01 | 45.2 | 0.01 | 0.01 | 0.07 | -0.3% | 12.2 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -3023 | 159.4 | -2806 | 76.3 | 125.0 | 2783 | -7.2% | 1.74 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁴ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 186.8 | 7.38 | 535.9 | 23.4 | 17.3 | 41.4 | +186.9% | 20.1 | **N better** |
| N vs P+ | q1_p99_us | 539.8 | 58.3 | 1430 | 224.0 | 163.7 | 192.0 | +165.0% | 5.44 | **N better** |
| N vs P+ | q2_p50_us | 147.9 | 7.21 | 419.4 | 12.5 | 10.2 | 11.0 | +183.5% | 26.7 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 29964 | 362.5 | 42502 | 964.9 | 728.9 | 960.4 | +41.8% | 17.2 | **N better** |
| N vs P+ | q6_p50_us | 45660 | 538.5 | 42416 | 1027 | 819.7 | 960.2 | -7.1% | 3.96 | no difference |
| N vs P+ | c2_q1_p50_us | 146.8 | 10.6 | 586.0 | 13.9 | 12.4 | 129.7 | +299.2% | 35.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 462.8 | 33.3 | 4753 | 400.9 | 284.4 | 6227 | +926.8% | 15.1 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 105.4 | 8.53 | 496.8 | 19.0 | 14.7 | 7.55 | +371.5% | 26.6 | **N better** |
| N vs P+ | c4_q1_p50_us | 248.9 | 13.2 | 1582 | 47.1 | 34.6 | 128.6 | +535.6% | 38.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1099 | 253.3 | 8334 | 548.3 | 427.0 | 1901 | +658.7% | 16.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 198.0 | 8.45 | 1395 | 68.3 | 48.6 | 163.7 | +604.6% | 24.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1177 | 447.8 | 5756 | 2220 | 1601 | 6944 | +389.0% | 2.86 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 591.6 | 36.2 | 757.4 | 29.0 | 32.8 | 66.3 | +28.0% | 5.06 | **N better** |
| N vs P+ | write_p99_us | 1065 | 99.2 | 2647 | 118.4 | 109.2 | 596.3 | +148.6% | 14.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1627 | 80.9 | 1062 | 32.3 | 61.6 | 106.2 | -34.7% | 9.16 | **N better** |
| N vs P+ | pss_mib | 88.6 | 1.33 | 45.4 | 0.03 | 0.94 | 0.40 | -48.8% | 46.1 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 19638 | 3238 | -2297 | 482.7 | 2315 | 43291 | -111.7% | 9.48 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 186.8 | 7.38 | 127.2 | 3.77 | 5.86 | 17.0 | -31.9% | 10.2 | **P better** |
| N vs P | q1_p99_us | 539.8 | 58.3 | 470.6 | 104.2 | 84.4 | 29.0 | -12.8% | 0.82 | no difference |
| N vs P | q2_p50_us | 147.9 | 7.21 | 214.7 | 15.2 | 11.9 | 23.4 | +45.1% | 5.61 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 29964 | 362.5 | 170.6 | 11.6 | 256.5 | 854.8 | -99.4% | 116.2 | **P better** |
| N vs P | q6_p50_us | 45660 | 538.5 | 2708 | 41.1 | 381.9 | 421.7 | -94.1% | 112.5 | **P better** |
| N vs P | c2_q1_p50_us | 146.8 | 10.6 | 128.9 | 11.2 | 10.9 | 35.7 | -12.2% | 1.63 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 462.8 | 33.3 | 1330 | 332.3 | 236.2 | 1511 | +187.4% | 3.67 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q2_p50_us | 105.4 | 8.53 | 240.7 | 29.2 | 21.5 | 21.1 | +128.4% | 6.29 | **N better** |
| N vs P | c4_q1_p50_us | 248.9 | 13.2 | 163.6 | 32.5 | 24.8 | 148.3 | -34.3% | 3.44 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1099 | 253.3 | 2322 | 274.3 | 264.0 | 785.0 | +111.3% | 4.63 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 198.0 | 8.45 | 327.7 | 33.7 | 24.6 | 103.7 | +65.5% | 5.28 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 1177 | 447.8 | 3292 | 389.4 | 419.6 | 1446 | +179.7% | 5.04 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 591.6 | 36.2 | 137050 | 497.4 | 352.6 | 8020 | +23067.8% | 387.0 | **N better** |
| N vs P | write_p99_us | 1065 | 99.2 | 161659 | 4783 | 3383 | 6373 | +15081.9% | 47.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1627 | 80.9 | 7.18 | 0.04 | 57.2 | 88.2 | -99.6% | 28.3 | **N better** |
| N vs P | pss_mib | 88.6 | 1.33 | 45.6 | 0.05 | 0.94 | 0.39 | -48.6% | 45.8 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 19638 | 3238 | -1190 | 266.2 | 2297 | 43063 | -106.1% | 9.07 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 535.9 | 23.4 | 127.2 | 3.77 | 16.7 | 38.2 | -76.3% | 24.4 | **P better** |
| P+ vs P | q1_p99_us | 1430 | 224.0 | 470.6 | 104.2 | 174.7 | 193.6 | -67.1% | 5.49 | **P better** |
| P+ vs P | q2_p50_us | 419.4 | 12.5 | 214.7 | 15.2 | 13.9 | 20.7 | -48.8% | 14.7 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 256.2 | 19.3 | 253.9 | 18.6 | 18.9 | 70.7 | -0.9% | 0.12 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 13066 | 190.7 | 794.5 | 9.86 | 135.1 | 547.3 | -93.9% | 90.9 | **P better** |
| P+ vs P | q5_p50_us | 42502 | 964.9 | 170.6 | 11.6 | 682.4 | 439.9 | -99.6% | 62.0 | **P better** |
| P+ vs P | q6_p50_us | 42416 | 1027 | 2708 | 41.1 | 726.4 | 876.0 | -93.6% | 54.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 586.0 | 13.9 | 128.9 | 11.2 | 12.6 | 129.5 | -78.0% | 36.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 4753 | 400.9 | 1330 | 332.3 | 368.2 | 6408 | -72.0% | 9.30 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 496.8 | 19.0 | 240.7 | 29.2 | 24.6 | 19.7 | -51.6% | 10.4 | **P better** |
| P+ vs P | c4_q1_p50_us | 1582 | 47.1 | 163.6 | 32.5 | 40.4 | 75.7 | -89.7% | 35.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8334 | 548.3 | 2322 | 274.3 | 433.5 | 1846 | -72.1% | 13.9 | **P better** |
| P+ vs P | c4_q2_p50_us | 1395 | 68.3 | 327.7 | 33.7 | 53.8 | 174.7 | -76.5% | 19.8 | **P better** |
| P+ vs P | c4_q2_p99_us | 5756 | 2220 | 3292 | 389.4 | 1594 | 7091 | -42.8% | 1.55 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 757.4 | 29.0 | 137050 | 497.4 | 352.3 | 8020 | +17994.6% | 386.9 | **P+ better** |
| P+ vs P | write_p99_us | 2647 | 118.4 | 161659 | 4783 | 3383 | 6353 | +6007.4% | 47.0 | **P+ better** |
| P+ vs P | commits_per_s | 1062 | 32.3 | 7.18 | 0.04 | 22.8 | 59.1 | -99.3% | 46.2 | **P+ better** |
| P+ vs P | pss_mib | 45.4 | 0.03 | 45.6 | 0.05 | 0.04 | 0.04 | +0.5% | 5.14 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -2297 | 482.7 | -1190 | 266.2 | 389.8 | 5823 | -48.2% | 2.84 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 127.2 | 3.77 | 131.3 | 4.52 | 4.16 | 4.28 | +3.2% | 0.98 | BELOW FLOOR |
| P vs M | q1_p99_us | 470.6 | 104.2 | 465.2 | 61.4 | 85.5 | 53.8 | -1.1% | 0.06 | BELOW FLOOR |
| P vs M | q2_p50_us | 214.7 | 15.2 | 218.9 | 6.85 | 11.8 | 23.3 | +2.0% | 0.36 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 253.9 | 18.6 | 254.7 | 19.4 | 19.0 | 54.4 | +0.3% | 0.04 | BELOW FLOOR |
| P vs M | q4_p50_us | 794.5 | 9.86 | 842.2 | 9.35 | 9.61 | 89.6 | +6.0% | 4.96 | BELOW FLOOR |
| P vs M | q5_p50_us | 170.6 | 11.6 | 162.4 | 9.74 | 10.7 | 39.0 | -4.8% | 0.76 | BELOW FLOOR |
| P vs M | q6_p50_us | 2708 | 41.1 | 2789 | 62.5 | 52.9 | 188.8 | +3.0% | 1.52 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 128.9 | 11.2 | 120.4 | 62.6 | 44.9 | 26.2 | -6.6% | 0.19 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1330 | 332.3 | 1485 | 541.7 | 449.4 | 1770 | +11.6% | 0.34 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c2_q2_p50_us | 240.7 | 29.2 | 220.6 | 57.1 | 45.3 | 28.9 | -8.3% | 0.44 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 163.6 | 32.5 | 143.4 | 27.8 | 30.2 | 115.7 | -12.3% | 0.67 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2322 | 274.3 | 2516 | 480.4 | 391.2 | 1759 | +8.4% | 0.50 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q2_p50_us | 327.7 | 33.7 | 311.2 | 52.2 | 43.9 | 98.1 | -5.0% | 0.38 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 3292 | 389.4 | 3333 | 365.7 | 377.8 | 1745 | +1.3% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 137050 | 497.4 | 138006 | 1316 | 995.0 | 8080 | +0.7% | 0.96 | BELOW FLOOR |
| P vs M | write_p99_us | 161659 | 4783 | 163868 | 10784 | 8342 | 8629 | +1.4% | 0.26 | BELOW FLOOR |
| P vs M | commits_per_s | 7.18 | 0.04 | 7.10 | 0.08 | 0.06 | 0.24 | -1.1% | 1.19 | BELOW FLOOR |
| P vs M | pss_mib | 45.6 | 0.05 | 45.4 | 0.02 | 0.04 | 0.12 | -0.5% | 5.55 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -1190 | 266.2 | -814.9 | 168.1 | 222.6 | 3456 | -31.5% | 1.69 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁴ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 185.4 | 13.8 | 551.0 | 16.9 | 15.4 | 11.5 | +197.2% | 23.7 | **N better** |
| N vs P+ | q1_p99_us | 581.0 | 73.2 | 1897 | 418.0 | 300.1 | 482.7 | +226.5% | 4.39 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 149.7 | 10.9 | 437.1 | 15.2 | 13.2 | 5.95 | +191.9% | 21.7 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 30631 | 544.1 | 43200 | 1039 | 829.4 | 1816 | +41.0% | 15.2 | **N better** |
| N vs P+ | q6_p50_us | 46507 | 846.4 | 43756 | 1360 | 1132 | 1779 | -5.9% | 2.43 | no difference |
| N vs P+ | c2_q1_p50_us | 139.4 | 8.77 | 574.9 | 21.6 | 16.5 | 39.9 | +312.4% | 26.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 423.4 | 136.2 | 4895 | 494.8 | 362.9 | 301.6 | +1056.0% | 12.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 91.4 | 4.88 | 506.0 | 14.1 | 10.6 | 123.7 | +453.6% | 39.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 248.3 | 9.69 | 1585 | 125.3 | 88.9 | 127.4 | +538.3% | 15.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 1052 | 129.4 | 8370 | 982.3 | 700.6 | 2044 | +695.8% | 10.4 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 192.3 | 10.7 | 1441 | 59.2 | 42.6 | 187.8 | +649.4% | 29.3 | **N better** |
| N vs P+ | c4_q2_p99_us | 860.2 | 193.5 | 7411 | 1005 | 723.5 | 2227 | +761.5% | 9.05 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 532.4 | 106.8 | 775.6 | 24.9 | 77.6 | 76.2 | +45.7% | 3.14 | **N better** |
| N vs P+ | write_p99_us | 969.4 | 225.1 | 2953 | 218.6 | 221.9 | 1335 | +204.6% | 8.94 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1768 | 170.5 | 1025 | 45.7 | 124.8 | 54.2 | -42.0% | 5.95 | **N better** |
| N vs P+ | pss_mib | 88.3 | 0.92 | 45.3 | 0.01 | 0.65 | 5.04 | -48.7% | 65.8 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 19719 | 4190 | -841.1 | 167.4 | 2965 | 37491 | -104.3% | 6.93 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 185.4 | 13.8 | 125.8 | 4.72 | 10.3 | 10.5 | -32.2% | 5.80 | **P better** |
| N vs P | q1_p99_us | 581.0 | 73.2 | 514.6 | 98.4 | 86.7 | 310.6 | -11.4% | 0.77 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 149.7 | 10.9 | 209.6 | 11.5 | 11.2 | 4.98 | +39.9% | 5.34 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 30631 | 544.1 | 168.1 | 13.7 | 384.8 | 915.4 | -99.5% | 79.2 | **P better** |
| N vs P | q6_p50_us | 46507 | 846.4 | 2823 | 45.1 | 599.3 | 1410 | -93.9% | 72.9 | **P better** |
| N vs P | c2_q1_p50_us | 139.4 | 8.77 | 65.0 | 15.0 | 12.3 | 41.1 | -53.3% | 6.04 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 423.4 | 136.2 | 1066 | 176.9 | 157.9 | 1629 | +151.7% | 4.07 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 91.4 | 4.88 | 152.5 | 19.9 | 14.5 | 67.5 | +66.8% | 4.21 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 248.3 | 9.69 | 175.9 | 34.2 | 25.1 | 78.0 | -29.2% | 2.89 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 1052 | 129.4 | 2576 | 386.3 | 288.1 | 1225 | +145.0% | 5.29 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p50_us | 192.3 | 10.7 | 358.7 | 52.3 | 37.7 | 169.6 | +86.5% | 4.41 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 860.2 | 193.5 | 2976 | 168.1 | 181.2 | 3509 | +245.9% | 11.7 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 532.4 | 106.8 | 142068 | 2784 | 1970 | 6096 | +26586.2% | 71.8 | **N better** |
| N vs P | write_p99_us | 969.4 | 225.1 | 169047 | 5372 | 3802 | 11055 | +17339.1% | 44.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1768 | 170.5 | 6.87 | 0.09 | 120.5 | 44.6 | -99.6% | 14.6 | **N better** |
| N vs P | pss_mib | 88.3 | 0.92 | 45.4 | 0.04 | 0.65 | 5.04 | -48.5% | 65.5 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 19719 | 4190 | -875.7 | 189.7 | 2966 | 37469 | -104.4% | 6.94 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 551.0 | 16.9 | 125.8 | 4.72 | 12.4 | 5.94 | -77.2% | 34.3 | **P better** |
| P+ vs P | q1_p99_us | 1897 | 418.0 | 514.6 | 98.4 | 303.7 | 478.4 | -72.9% | 4.55 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 437.1 | 15.2 | 209.6 | 11.5 | 13.5 | 6.22 | -52.1% | 16.9 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 265.1 | 20.3 | 255.8 | 38.8 | 30.9 | 20.8 | -3.5% | 0.30 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 12688 | 305.9 | 814.3 | 26.8 | 217.1 | 330.5 | -93.6% | 54.7 | **P better** |
| P+ vs P | q5_p50_us | 43200 | 1039 | 168.1 | 13.7 | 734.8 | 1568 | -99.6% | 58.6 | **P better** |
| P+ vs P | q6_p50_us | 43756 | 1360 | 2823 | 45.1 | 961.9 | 1644 | -93.5% | 42.6 | **P better** |
| P+ vs P | c2_q1_p50_us | 574.9 | 21.6 | 65.0 | 15.0 | 18.6 | 13.2 | -88.7% | 27.4 | **P better** |
| P+ vs P | c2_q1_p99_us | 4895 | 494.8 | 1066 | 176.9 | 371.6 | 1606 | -78.2% | 10.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 506.0 | 14.1 | 152.5 | 19.9 | 17.3 | 124.6 | -69.9% | 20.5 | **P better** |
| P+ vs P | c4_q1_p50_us | 1585 | 125.3 | 175.9 | 34.2 | 91.9 | 148.9 | -88.9% | 15.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8370 | 982.3 | 2576 | 386.3 | 746.4 | 2196 | -69.2% | 7.76 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1441 | 59.2 | 358.7 | 52.3 | 55.9 | 245.1 | -75.1% | 19.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 7411 | 1005 | 2976 | 168.1 | 720.3 | 3132 | -59.8% | 6.16 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 775.6 | 24.9 | 142068 | 2784 | 1969 | 6096 | +18216.1% | 71.8 | **P+ better** |
| P+ vs P | write_p99_us | 2953 | 218.6 | 169047 | 5372 | 3802 | 10975 | +5625.4% | 43.7 | **P+ better** |
| P+ vs P | commits_per_s | 1025 | 45.7 | 6.87 | 0.09 | 32.3 | 30.9 | -99.3% | 31.5 | **P+ better** |
| P+ vs P | pss_mib | 45.3 | 0.01 | 45.4 | 0.04 | 0.03 | 0.03 | +0.3% | 5.35 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -841.1 | 167.4 | -875.7 | 189.7 | 178.9 | 2504 | +4.1% | 0.19 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 125.8 | 4.72 | 129.4 | 3.43 | 4.13 | 11.1 | +2.9% | 0.88 | BELOW FLOOR |
| P vs M | q1_p99_us | 514.6 | 98.4 | 531.6 | 132.3 | 116.6 | 250.6 | +3.3% | 0.15 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 209.6 | 11.5 | 218.2 | 7.91 | 9.85 | 29.9 | +4.1% | 0.88 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 255.8 | 38.8 | 275.2 | 16.9 | 29.9 | 33.3 | +7.6% | 0.65 | BELOW FLOOR |
| P vs M | q4_p50_us | 814.3 | 26.8 | 824.8 | 35.6 | 31.5 | 46.9 | +1.3% | 0.33 | BELOW FLOOR |
| P vs M | q5_p50_us | 168.1 | 13.7 | 173.6 | 5.94 | 10.5 | 28.1 | +3.3% | 0.53 | BELOW FLOOR |
| P vs M | q6_p50_us | 2823 | 45.1 | 2823 | 86.7 | 69.1 | 1202 | +0.0% | 0.01 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 65.0 | 15.0 | 52.5 | 1.10 | 10.7 | 98.5 | -19.3% | 1.18 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1066 | 176.9 | 1010 | 31.6 | 127.1 | 1664 | -5.2% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 152.5 | 19.9 | 135.1 | 9.25 | 15.5 | 157.0 | -11.4% | 1.12 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 175.9 | 34.2 | 133.5 | 39.2 | 36.8 | 78.5 | -24.1% | 1.15 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2576 | 386.3 | 2195 | 572.4 | 488.3 | 1170 | -14.8% | 0.78 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 358.7 | 52.3 | 293.5 | 30.5 | 42.8 | 164.2 | -18.2% | 1.52 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 2976 | 168.1 | 3106 | 857.7 | 618.0 | 4340 | +4.4% | 0.21 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 142068 | 2784 | 141898 | 1184 | 2139 | 8359 | -0.1% | 0.08 | BELOW FLOOR |
| P vs M | write_p99_us | 169047 | 5372 | 169415 | 10048 | 8057 | 15014 | +0.2% | 0.05 | BELOW FLOOR |
| P vs M | commits_per_s | 6.87 | 0.09 | 6.88 | 0.11 | 0.10 | 0.32 | +0.2% | 0.15 | BELOW FLOOR |
| P vs M | pss_mib | 45.4 | 0.04 | 45.3 | 0.06 | 0.05 | 0.03 | -0.2% | 2.02 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -875.7 | 189.7 | -854.4 | 195.6 | 192.7 | 2334 | -2.4% | 0.11 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁴ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 187.1 | 10.6 | 565.0 | 26.7 | 20.3 | 21.8 | +202.0% | 18.6 | **N better** |
| N vs P+ | q1_p99_us | 580.4 | 80.8 | 1571 | 182.2 | 140.9 | 87.8 | +170.7% | 7.03 | **N better** |
| N vs P+ | q2_p50_us | 152.4 | 9.02 | 449.2 | 32.7 | 24.0 | 44.2 | +194.8% | 12.4 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 30179 | 136.6 | 44052 | 1178 | 838.6 | 900.5 | +46.0% | 16.5 | **N better** |
| N vs P+ | q6_p50_us | 45314 | 341.4 | 44798 | 608.4 | 493.3 | 1985 | -1.1% | 1.04 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 125.1 | 12.2 | 584.6 | 38.2 | 28.4 | 44.6 | +367.3% | 16.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 336.3 | 53.6 | 5492 | 410.8 | 292.9 | 196.9 | +1533.2% | 17.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 90.9 | 6.40 | 525.0 | 41.4 | 29.7 | 28.0 | +477.4% | 14.6 | **N better** |
| N vs P+ | c4_q1_p50_us | 239.6 | 28.5 | 1657 | 114.3 | 83.3 | 279.3 | +591.4% | 17.0 | **N better** |
| N vs P+ | c4_q1_p99_us | 1032 | 154.2 | 8302 | 2349 | 1665 | 5454 | +704.6% | 4.37 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 197.4 | 18.2 | 1450 | 35.4 | 28.1 | 126.3 | +634.4% | 44.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 2081 | 700.3 | 7713 | 1489 | 1164 | 2967 | +270.6% | 4.84 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 520.6 | 52.3 | 795.6 | 39.4 | 46.3 | 21.3 | +52.8% | 5.94 | **N better** |
| N vs P+ | write_p99_us | 925.0 | 108.1 | 2676 | 49.4 | 84.0 | 254.7 | +189.2% | 20.8 | **N better** |
| N vs P+ | commits_per_s | 1831 | 105.7 | 1042 | 18.6 | 75.9 | 20.9 | -43.1% | 10.4 | **N better** |
| N vs P+ | pss_mib | 88.0 | 1.83 | 45.2 | 0.02 | 1.30 | 0.16 | -48.7% | 33.0 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 21784 | 3705 | -929.4 | 194.2 | 2624 | 39869 | -104.3% | 8.66 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 187.1 | 10.6 | 130.8 | 3.51 | 7.88 | 21.2 | -30.1% | 7.15 | **P better** |
| N vs P | q1_p99_us | 580.4 | 80.8 | 456.5 | 50.3 | 67.3 | 95.2 | -21.3% | 1.84 | no difference |
| N vs P | q2_p50_us | 152.4 | 9.02 | 219.5 | 7.50 | 8.30 | 9.75 | +44.1% | 8.09 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 30179 | 136.6 | 187.2 | 11.5 | 96.9 | 198.5 | -99.4% | 309.4 | **P better** |
| N vs P | q6_p50_us | 45314 | 341.4 | 2919 | 98.9 | 251.3 | 563.5 | -93.6% | 168.7 | **P better** |
| N vs P | c2_q1_p50_us | 125.1 | 12.2 | 123.3 | 8.93 | 10.7 | 67.2 | -1.5% | 0.17 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q1_p99_us | 336.3 | 53.6 | 1516 | 718.0 | 509.1 | 919.8 | +350.7% | 2.32 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 90.9 | 6.40 | 235.6 | 34.8 | 25.0 | 82.3 | +159.1% | 5.79 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 239.6 | 28.5 | 190.4 | 47.6 | 39.2 | 20.9 | -20.5% | 1.25 | no difference |
| N vs P | c4_q1_p99_us | 1032 | 154.2 | 2503 | 262.7 | 215.4 | 775.5 | +142.6% | 6.83 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 197.4 | 18.2 | 366.0 | 20.0 | 19.2 | 100.3 | +85.4% | 8.80 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p99_us | 2081 | 700.3 | 4399 | 1271 | 1026 | 1415 | +111.3% | 2.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 520.6 | 52.3 | 139382 | 2266 | 1603 | 895.7 | +26673.2% | 86.6 | **N better** |
| N vs P | write_p99_us | 925.0 | 108.1 | 190725 | 6997 | 4948 | 20710 | +20517.9% | 38.4 | **N better** |
| N vs P | commits_per_s | 1831 | 105.7 | 6.94 | 0.09 | 74.7 | 17.0 | -99.6% | 24.4 | **N better** |
| N vs P | pss_mib | 88.0 | 1.83 | 45.6 | 0.04 | 1.30 | 0.15 | -48.2% | 32.7 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 21784 | 3705 | -869.5 | 174.1 | 2623 | 39851 | -104.0% | 8.64 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 565.0 | 26.7 | 130.8 | 3.51 | 19.0 | 5.16 | -76.9% | 22.8 | **P better** |
| P+ vs P | q1_p99_us | 1571 | 182.2 | 456.5 | 50.3 | 133.7 | 37.3 | -70.9% | 8.34 | **P better** |
| P+ vs P | q2_p50_us | 449.2 | 32.7 | 219.5 | 7.50 | 23.7 | 43.2 | -51.1% | 9.68 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 282.2 | 39.7 | 255.2 | 20.8 | 31.7 | 39.5 | -9.6% | 0.85 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 12901 | 432.2 | 825.5 | 12.4 | 305.7 | 615.8 | -93.6% | 39.5 | **P better** |
| P+ vs P | q5_p50_us | 44052 | 1178 | 187.2 | 11.5 | 833.1 | 878.7 | -99.6% | 52.7 | **P better** |
| P+ vs P | q6_p50_us | 44798 | 608.4 | 2919 | 98.9 | 435.8 | 1915 | -93.5% | 96.1 | **P better** |
| P+ vs P | c2_q1_p50_us | 584.6 | 38.2 | 123.3 | 8.93 | 27.8 | 63.6 | -78.9% | 16.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5492 | 410.8 | 1516 | 718.0 | 584.9 | 915.6 | -72.4% | 6.80 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q2_p50_us | 525.0 | 41.4 | 235.6 | 34.8 | 38.3 | 86.2 | -55.1% | 7.56 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1657 | 114.3 | 190.4 | 47.6 | 87.5 | 279.9 | -88.5% | 16.8 | **P better** |
| P+ vs P | c4_q1_p99_us | 8302 | 2349 | 2503 | 262.7 | 1671 | 5402 | -69.8% | 3.47 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1450 | 35.4 | 366.0 | 20.0 | 28.7 | 79.6 | -74.8% | 37.7 | **P better** |
| P+ vs P | c4_q2_p99_us | 7713 | 1489 | 4399 | 1271 | 1384 | 3245 | -43.0% | 2.39 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 795.6 | 39.4 | 139382 | 2266 | 1602 | 896.0 | +17420.0% | 86.5 | **P+ better** |
| P+ vs P | write_p99_us | 2676 | 49.4 | 190725 | 6997 | 4948 | 20711 | +7028.1% | 38.0 | **P+ better** |
| P+ vs P | commits_per_s | 1042 | 18.6 | 6.94 | 0.09 | 13.1 | 12.1 | -99.3% | 78.8 | **P+ better** |
| P+ vs P | pss_mib | 45.2 | 0.02 | 45.6 | 0.04 | 0.03 | 0.05 | +0.9% | 13.1 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -929.4 | 194.2 | -869.5 | 174.1 | 184.4 | 3082 | -6.4% | 0.32 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 130.8 | 3.51 | 135.5 | 5.22 | 4.45 | 17.1 | +3.6% | 1.06 | BELOW FLOOR |
| P vs M | q1_p99_us | 456.5 | 50.3 | 445.0 | 42.7 | 46.6 | 45.5 | -2.5% | 0.25 | BELOW FLOOR |
| P vs M | q2_p50_us | 219.5 | 7.50 | 217.7 | 12.9 | 10.6 | 11.4 | -0.8% | 0.17 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 255.2 | 20.8 | 256.2 | 12.8 | 17.3 | 32.2 | +0.4% | 0.06 | BELOW FLOOR |
| P vs M | q4_p50_us | 825.5 | 12.4 | 816.8 | 35.7 | 26.7 | 57.1 | -1.1% | 0.33 | BELOW FLOOR |
| P vs M | q5_p50_us | 187.2 | 11.5 | 174.0 | 11.1 | 11.3 | 20.2 | -7.1% | 1.17 | BELOW FLOOR |
| P vs M | q6_p50_us | 2919 | 98.9 | 2910 | 52.5 | 79.1 | 482.7 | -0.3% | 0.11 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 123.3 | 8.93 | 117.8 | 19.1 | 14.9 | 57.3 | -4.4% | 0.37 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1516 | 718.0 | 1543 | 574.5 | 650.2 | 935.6 | +1.8% | 0.04 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 235.6 | 34.8 | 210.5 | 44.4 | 39.9 | 91.8 | -10.7% | 0.63 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 190.4 | 47.6 | 141.0 | 19.5 | 36.4 | 66.0 | -26.0% | 1.36 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 2503 | 262.7 | 2985 | 309.0 | 286.8 | 538.0 | +19.3% | 1.68 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 366.0 | 20.0 | 267.8 | 23.4 | 21.8 | 34.9 | -26.9% | 4.52 | **M better** |
| P vs M | c4_q2_p99_us | 4399 | 1271 | 3197 | 578.0 | 987.0 | 1366 | -27.3% | 1.22 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 139382 | 2266 | 139833 | 1763 | 2030 | 5647 | +0.3% | 0.22 | BELOW FLOOR |
| P vs M | write_p99_us | 190725 | 6997 | 172119 | 14653 | 11482 | 37929 | -9.8% | 1.62 | BELOW FLOOR |
| P vs M | commits_per_s | 6.94 | 0.09 | 6.95 | 0.14 | 0.12 | 0.42 | +0.2% | 0.13 | BELOW FLOOR |
| P vs M | pss_mib | 45.6 | 0.04 | 45.5 | 0.05 | 0.05 | 0.21 | -0.1% | 0.71 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | -869.5 | 174.1 | -810.9 | 175.1 | 174.6 | 2707 | -6.7% | 0.34 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁴ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 192.8 | 18.2 | 584.5 | 12.6 | 15.7 | 67.2 | +203.2% | 25.0 | **N better** |
| N vs P+ | q1_p99_us | 626.5 | 71.0 | 1798 | 361.9 | 260.8 | 713.5 | +187.0% | 4.49 | **N better** |
| N vs P+ | q2_p50_us | 152.0 | 4.38 | 454.1 | 6.89 | 5.78 | 26.5 | +198.7% | 52.3 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 30443 | 695.0 | 42415 | 944.4 | 829.2 | 3644 | +39.3% | 14.4 | **N better** |
| N vs P+ | q6_p50_us | 46347 | 1205 | 43817 | 978.0 | 1098 | 3593 | -5.5% | 2.31 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 137.4 | 22.8 | 598.9 | 33.1 | 28.4 | 147.8 | +335.8% | 16.2 | **N better** |
| N vs P+ | c2_q1_p99_us | 319.4 | 61.7 | 6132 | 1221 | 864.3 | 411.9 | +1820.0% | 6.73 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 92.0 | 10.4 | 502.4 | 30.1 | 22.5 | 127.1 | +446.2% | 18.2 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 265.0 | 22.1 | 1657 | 55.8 | 42.4 | 282.9 | +525.4% | 32.8 | **N better** |
| N vs P+ | c4_q1_p99_us | 987.0 | 247.0 | 8262 | 1235 | 890.9 | 11276 | +737.1% | 8.17 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c4_q2_p50_us | 188.7 | 21.7 | 1457 | 76.9 | 56.5 | 184.7 | +671.9% | 22.4 | **N better** |
| N vs P+ | c4_q2_p99_us | 1535 | 570.9 | 7284 | 1600 | 1202 | 8797 | +374.4% | 4.78 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 547.8 | 35.2 | 813.2 | 35.3 | 35.3 | 36.0 | +48.5% | 7.53 | **N better** |
| N vs P+ | write_p99_us | 922.8 | 90.7 | 2979 | 223.4 | 170.5 | 465.1 | +222.8% | 12.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1774 | 195.2 | 1014 | 33.4 | 140.0 | 97.5 | -42.8% | 5.43 | **N better** |
| N vs P+ | pss_mib | 86.6 | 0.26 | 45.2 | 0.02 | 0.18 | 5.17 | -47.7% | 224.5 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 20473 | 3925 | -890.3 | 175.7 | 2778 | 38418 | -104.3% | 7.69 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 192.8 | 18.2 | 127.6 | 2.07 | 13.0 | 21.7 | -33.8% | 5.03 | **P better** |
| N vs P | q1_p99_us | 626.5 | 71.0 | 484.4 | 50.2 | 61.5 | 57.1 | -22.7% | 2.31 | no difference |
| N vs P | q2_p50_us | 152.0 | 4.38 | 213.1 | 6.07 | 5.30 | 15.3 | +40.1% | 11.5 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 30443 | 695.0 | 184.8 | 23.8 | 491.7 | 2912 | -99.4% | 61.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | 46347 | 1205 | 2773 | 92.8 | 854.8 | 344.0 | -94.0% | 51.0 | **P better** |
| N vs P | c2_q1_p50_us | 137.4 | 22.8 | 51.7 | 1.39 | 16.2 | 46.3 | -62.4% | 5.31 | **P better** |
| N vs P | c2_q1_p99_us | 319.4 | 61.7 | 1046 | 82.0 | 72.5 | 450.1 | +227.5% | 10.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 92.0 | 10.4 | 145.5 | 23.2 | 18.0 | 88.8 | +58.2% | 2.98 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 265.0 | 22.1 | 144.0 | 8.11 | 16.7 | 95.2 | -45.6% | 7.26 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 987.0 | 247.0 | 2091 | 363.3 | 310.7 | 1103 | +111.8% | 3.55 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p50_us | 188.7 | 21.7 | 275.3 | 21.1 | 21.4 | 173.2 | +45.9% | 4.05 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 1535 | 570.9 | 3067 | 648.0 | 610.7 | 1408 | +99.7% | 2.51 | no difference |
| N vs P | write_p50_us | 547.8 | 35.2 | 138316 | 1327 | 938.9 | 4404 | +25151.0% | 146.7 | **N better** |
| N vs P | write_p99_us | 922.8 | 90.7 | 169018 | 4872 | 3445 | 17564 | +18215.9% | 48.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1774 | 195.2 | 7.10 | 0.04 | 138.0 | 58.0 | -99.6% | 12.8 | **N better** |
| N vs P | pss_mib | 86.6 | 0.26 | 45.4 | 0.03 | 0.18 | 5.17 | -47.6% | 223.2 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 20473 | 3925 | -744.4 | 181.4 | 2778 | 38373 | -103.6% | 7.64 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 584.5 | 12.6 | 127.6 | 2.07 | 9.05 | 64.5 | -78.2% | 50.5 | **P better** |
| P+ vs P | q1_p99_us | 1798 | 361.9 | 484.4 | 50.2 | 258.4 | 714.3 | -73.1% | 5.08 | **P better** |
| P+ vs P | q2_p50_us | 454.1 | 6.89 | 213.1 | 6.07 | 6.50 | 23.2 | -53.1% | 37.1 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 313.2 | 30.2 | 258.6 | 14.8 | 23.8 | 121.2 | -17.4% | 2.30 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 12790 | 247.6 | 813.3 | 15.5 | 175.4 | 287.2 | -93.6% | 68.3 | **P better** |
| P+ vs P | q5_p50_us | 42415 | 944.4 | 184.8 | 23.8 | 668.0 | 2193 | -99.6% | 63.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | 43817 | 978.0 | 2773 | 92.8 | 694.7 | 3604 | -93.7% | 59.1 | **P better** |
| P+ vs P | c2_q1_p50_us | 598.9 | 33.1 | 51.7 | 1.39 | 23.4 | 140.6 | -91.4% | 23.4 | **P better** |
| P+ vs P | c2_q1_p99_us | 6132 | 1221 | 1046 | 82.0 | 865.1 | 188.6 | -82.9% | 5.88 | **P better** |
| P+ vs P | c2_q2_p50_us | 502.4 | 30.1 | 145.5 | 23.2 | 26.9 | 98.5 | -71.0% | 13.3 | **P better** |
| P+ vs P | c4_q1_p50_us | 1657 | 55.8 | 144.0 | 8.11 | 39.9 | 285.6 | -91.3% | 38.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 8262 | 1235 | 2091 | 363.3 | 910.6 | 11329 | -74.7% | 6.78 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q2_p50_us | 1457 | 76.9 | 275.3 | 21.1 | 56.4 | 253.1 | -81.1% | 21.0 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 7284 | 1600 | 3067 | 648.0 | 1221 | 8873 | -57.9% | 3.45 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 813.2 | 35.3 | 138316 | 1327 | 938.9 | 4405 | +16908.5% | 146.4 | **P+ better** |
| P+ vs P | write_p99_us | 2979 | 223.4 | 169018 | 4872 | 3448 | 17562 | +5573.5% | 48.1 | **P+ better** |
| P+ vs P | commits_per_s | 1014 | 33.4 | 7.10 | 0.04 | 23.6 | 78.3 | -99.3% | 42.6 | **P+ better** |
| P+ vs P | pss_mib | 45.2 | 0.02 | 45.4 | 0.03 | 0.02 | 0.12 | +0.2% | 5.01 | BELOW FLOOR |
| P+ vs P | pss_growth_bytes_per_key_read | -890.3 | 175.7 | -744.4 | 181.4 | 178.6 | 2971 | -16.4% | 0.82 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 127.6 | 2.07 | 130.1 | 3.88 | 3.11 | 8.44 | +2.0% | 0.81 | BELOW FLOOR |
| P vs M | q1_p99_us | 484.4 | 50.2 | 489.8 | 111.1 | 86.2 | 114.8 | +1.1% | 0.06 | BELOW FLOOR |
| P vs M | q2_p50_us | 213.1 | 6.07 | 227.4 | 12.5 | 9.80 | 25.3 | +6.7% | 1.46 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 258.6 | 14.8 | 265.0 | 34.9 | 26.8 | 117.1 | +2.4% | 0.24 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q4_p50_us | 813.3 | 15.5 | 804.1 | 29.0 | 23.2 | 35.1 | -1.1% | 0.40 | BELOW FLOOR |
| P vs M | q5_p50_us | 184.8 | 23.8 | 180.9 | 14.1 | 19.5 | 74.4 | -2.1% | 0.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | 2773 | 92.8 | 2931 | 186.5 | 147.3 | 580.0 | +5.7% | 1.07 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 51.7 | 1.39 | 52.3 | 2.33 | 1.92 | 7.84 | +1.3% | 0.35 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1046 | 82.0 | 1121 | 162.6 | 128.8 | 213.7 | +7.2% | 0.59 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 145.5 | 23.2 | 158.0 | 35.6 | 30.0 | 47.6 | +8.6% | 0.42 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 144.0 | 8.11 | 163.0 | 29.4 | 21.6 | 72.8 | +13.2% | 0.88 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2091 | 363.3 | 2528 | 641.7 | 521.4 | 1100 | +20.9% | 0.84 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 275.3 | 21.1 | 302.6 | 61.2 | 45.8 | 174.0 | +9.9% | 0.60 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 3067 | 648.0 | 2982 | 683.4 | 665.9 | 1345 | -2.8% | 0.13 | BELOW FLOOR |
| P vs M | write_p50_us | 138316 | 1327 | 138493 | 1845 | 1607 | 4701 | +0.1% | 0.11 | BELOW FLOOR |
| P vs M | write_p99_us | 169018 | 4872 | 165369 | 12219 | 9301 | 20875 | -2.2% | 0.39 | BELOW FLOOR |
| P vs M | commits_per_s | 7.10 | 0.04 | 7.03 | 0.10 | 0.07 | 0.12 | -1.0% | 0.92 | BELOW FLOOR |
| P vs M | pss_mib | 45.4 | 0.03 | 45.5 | 0.06 | 0.05 | 0.11 | +0.3% | 2.49 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -744.4 | 181.4 | -753.6 | 277.2 | 234.3 | 2601 | +1.2% | 0.04 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁴ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 197.2 | 13.3 | 556.8 | 21.2 | 17.7 | 36.2 | +182.4% | 20.3 | **N better** |
| N vs P+ | q1_p99_us | 554.6 | 39.9 | 2344 | 862.2 | 610.3 | 488.0 | +322.7% | 2.93 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 147.8 | 7.01 | 447.7 | 23.6 | 17.4 | 44.8 | +202.9% | 17.2 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 30287 | 87.0 | 43892 | 1470 | 1041 | 1580 | +44.9% | 13.1 | **N better** |
| N vs P+ | q6_p50_us | 45932 | 595.9 | 44157 | 741.2 | 672.5 | 715.5 | -3.9% | 2.64 | no difference |
| N vs P+ | c2_q1_p50_us | 158.6 | 6.70 | 612.6 | 54.3 | 38.7 | 26.1 | +286.3% | 11.7 | **N better** |
| N vs P+ | c2_q1_p99_us | 501.0 | 58.9 | 5722 | 703.9 | 499.4 | 586.8 | +1042.1% | 10.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 100.4 | 13.8 | 539.8 | 44.2 | 32.7 | 68.9 | +437.5% | 13.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 267.0 | 11.6 | 1778 | 151.8 | 107.7 | 631.6 | +566.1% | 14.0 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q1_p99_us | 1060 | 202.0 | 7330 | 1601 | 1141 | 804.9 | +591.3% | 5.50 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 200.1 | 5.34 | 1629 | 127.6 | 90.3 | 294.3 | +714.0% | 15.8 | **N better** |
| N vs P+ | c4_q2_p99_us | 1662 | 399.9 | 8360 | 1193 | 889.4 | 3960 | +402.9% | 7.53 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 508.2 | 94.1 | 775.8 | 44.9 | 73.8 | 25.1 | +52.7% | 3.63 | **N better** |
| N vs P+ | write_p99_us | 1065 | 307.2 | 2942 | 225.7 | 269.5 | 438.2 | +176.1% | 6.96 | **N better** |
| N vs P+ | commits_per_s | 1691 | 245.2 | 993.7 | 64.4 | 179.3 | 345.1 | -41.2% | 3.89 | **N better** |
| N vs P+ | pss_mib | 85.1 | 3.12 | 45.4 | 0.04 | 2.21 | 0.40 | -46.6% | 18.0 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 20540 | 3800 | -889.2 | 204.8 | 2691 | 38510 | -104.3% | 7.96 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 197.2 | 13.3 | 130.2 | 3.42 | 9.68 | 36.1 | -34.0% | 6.92 | **P better** |
| N vs P | q1_p99_us | 554.6 | 39.9 | 449.1 | 40.3 | 40.1 | 65.4 | -19.0% | 2.63 | no difference |
| N vs P | q2_p50_us | 147.8 | 7.01 | 217.3 | 3.41 | 5.51 | 8.35 | +47.0% | 12.6 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 30287 | 87.0 | 168.6 | 15.0 | 62.4 | 547.5 | -99.4% | 482.4 | **P better** |
| N vs P | q6_p50_us | 45932 | 595.9 | 2876 | 55.6 | 423.2 | 530.6 | -93.7% | 101.7 | **P better** |
| N vs P | c2_q1_p50_us | 158.6 | 6.70 | 52.9 | 2.52 | 5.06 | 41.9 | -66.6% | 20.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c2_q1_p99_us | 501.0 | 58.9 | 1101 | 164.3 | 123.4 | 656.9 | +119.7% | 4.86 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 100.4 | 13.8 | 144.0 | 10.7 | 12.3 | 69.4 | +43.3% | 3.53 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 267.0 | 11.6 | 146.8 | 34.7 | 25.9 | 113.8 | -45.0% | 4.64 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1060 | 202.0 | 2481 | 612.2 | 455.9 | 425.0 | +134.0% | 3.12 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 200.1 | 5.34 | 270.7 | 25.6 | 18.5 | 76.5 | +35.3% | 3.81 | BELOW FLOOR |
| N vs P | c4_q2_p99_us | 1662 | 399.9 | 3642 | 641.8 | 534.7 | 1412 | +119.1% | 3.70 | **N better** |
| N vs P | write_p50_us | 508.2 | 94.1 | 137439 | 1870 | 1324 | 3893 | +26945.7% | 103.4 | **N better** |
| N vs P | write_p99_us | 1065 | 307.2 | 158790 | 10895 | 7707 | 12093 | +14804.3% | 20.5 | **N better** |
| N vs P | commits_per_s | 1691 | 245.2 | 7.15 | 0.13 | 173.4 | 295.4 | -99.6% | 9.71 | **N better** |
| N vs P | pss_mib | 85.1 | 3.12 | 45.2 | 0.02 | 2.21 | 0.40 | -46.9% | 18.1 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 20540 | 3800 | -861.8 | 240.9 | 2692 | 38499 | -104.2% | 7.95 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 556.8 | 21.2 | 130.2 | 3.42 | 15.2 | 7.75 | -76.6% | 28.0 | **P better** |
| P+ vs P | q1_p99_us | 2344 | 862.2 | 449.1 | 40.3 | 610.3 | 486.0 | -80.8% | 3.11 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 447.7 | 23.6 | 217.3 | 3.41 | 16.9 | 44.4 | -51.5% | 13.6 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 299.6 | 47.7 | 243.8 | 21.8 | 37.1 | 69.5 | -18.6% | 1.51 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 13266 | 396.3 | 827.4 | 28.6 | 281.0 | 290.6 | -93.8% | 44.3 | **P better** |
| P+ vs P | q5_p50_us | 43892 | 1470 | 168.6 | 15.0 | 1040 | 1482 | -99.6% | 42.1 | **P better** |
| P+ vs P | q6_p50_us | 44157 | 741.2 | 2876 | 55.6 | 525.5 | 883.8 | -93.5% | 78.5 | **P better** |
| P+ vs P | c2_q1_p50_us | 612.6 | 54.3 | 52.9 | 2.52 | 38.4 | 36.9 | -91.4% | 14.6 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c2_q1_p99_us | 5722 | 703.9 | 1101 | 164.3 | 511.1 | 595.6 | -80.8% | 9.04 | **P better** |
| P+ vs P | c2_q2_p50_us | 539.8 | 44.2 | 144.0 | 10.7 | 32.1 | 38.1 | -73.3% | 12.3 | **P better** |
| P+ vs P | c4_q1_p50_us | 1778 | 151.8 | 146.8 | 34.7 | 110.1 | 624.8 | -91.7% | 14.8 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c4_q1_p99_us | 7330 | 1601 | 2481 | 612.2 | 1212 | 684.2 | -66.2% | 4.00 | **P better** |
| P+ vs P | c4_q2_p50_us | 1629 | 127.6 | 270.7 | 25.6 | 92.0 | 295.7 | -83.4% | 14.8 | **P better** |
| P+ vs P | c4_q2_p99_us | 8360 | 1193 | 3642 | 641.8 | 957.6 | 4181 | -56.4% | 4.93 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 775.8 | 44.9 | 137439 | 1870 | 1323 | 3893 | +17616.0% | 103.3 | **P+ better** |
| P+ vs P | write_p99_us | 2942 | 225.7 | 158790 | 10895 | 7706 | 12094 | +5298.0% | 20.2 | **P+ better** |
| P+ vs P | commits_per_s | 993.7 | 64.4 | 7.15 | 0.13 | 45.5 | 178.5 | -99.3% | 21.7 | **P+ better** |
| P+ vs P | pss_mib | 45.4 | 0.04 | 45.2 | 0.02 | 0.03 | 0.04 | -0.6% | 7.80 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | -889.2 | 204.8 | -861.8 | 240.9 | 223.6 | 2778 | -3.1% | 0.12 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 130.2 | 3.42 | 129.0 | 5.36 | 4.50 | 11.9 | -0.9% | 0.27 | BELOW FLOOR |
| P vs M | q1_p99_us | 449.1 | 40.3 | 556.3 | 145.0 | 106.4 | 98.2 | +23.8% | 1.01 | no difference |
| P vs M | q2_p50_us | 217.3 | 3.41 | 215.1 | 16.5 | 11.9 | 6.54 | -1.0% | 0.18 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 243.8 | 21.8 | 227.9 | 25.0 | 23.4 | 9.60 | -6.5% | 0.68 | no difference |
| P vs M | q4_p50_us | 827.4 | 28.6 | 828.1 | 40.2 | 34.9 | 41.0 | +0.1% | 0.02 | BELOW FLOOR |
| P vs M | q5_p50_us | 168.6 | 15.0 | 193.4 | 13.3 | 14.2 | 30.1 | +14.7% | 1.74 | BELOW FLOOR |
| P vs M | q6_p50_us | 2876 | 55.6 | 2856 | 64.1 | 60.0 | 527.8 | -0.7% | 0.34 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 52.9 | 2.52 | 95.7 | 38.4 | 27.2 | 35.1 | +80.8% | 1.57 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q1_p99_us | 1101 | 164.3 | 1285 | 285.8 | 233.1 | 509.8 | +16.8% | 0.79 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 144.0 | 10.7 | 229.0 | 38.3 | 28.1 | 52.7 | +59.1% | 3.02 | **P better** |
| P vs M | c4_q1_p50_us | 146.8 | 34.7 | 172.0 | 29.2 | 32.1 | 47.1 | +17.1% | 0.78 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 2481 | 612.2 | 2728 | 403.1 | 518.3 | 224.9 | +10.0% | 0.48 | no difference |
| P vs M | c4_q2_p50_us | 270.7 | 25.6 | 334.8 | 39.2 | 33.1 | 57.9 | +23.6% | 1.93 | no difference |
| P vs M | c4_q2_p99_us | 3642 | 641.8 | 3544 | 785.3 | 717.1 | 1935 | -2.7% | 0.14 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | write_p50_us | 137439 | 1870 | 138501 | 1307 | 1613 | 3912 | +0.8% | 0.66 | BELOW FLOOR |
| P vs M | write_p99_us | 158790 | 10895 | 169443 | 16449 | 13952 | 19326 | +6.7% | 0.76 | BELOW FLOOR |
| P vs M | commits_per_s | 7.15 | 0.13 | 6.98 | 0.12 | 0.13 | 0.06 | -2.4% | 1.34 | no difference |
| P vs M | pss_mib | 45.2 | 0.02 | 45.3 | 0.03 | 0.03 | 0.02 | +0.2% | 3.80 | no difference |
| P vs M | pss_growth_bytes_per_key_read | -861.8 | 240.9 | -882.3 | 215.8 | 228.7 | 2816 | +2.4% | 0.09 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁵ accounts, seed 1

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 131.6 | 11.0 | 507.6 | 18.4 | 15.2 | 97.2 | +285.6% | 24.8 | **N better** |
| N vs P+ | q1_p99_us | 958.0 | 480.0 | 2352 | 466.0 | 473.0 | 2060 | +145.5% | 2.95 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 156.2 | 9.98 | 458.4 | 33.1 | 24.5 | 2.04 | +193.5% | 12.4 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 606711 | 21094 | 930441 | 26551 | 23979 | 26342 | +53.4% | 13.5 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 65.6 | 5.70 | 528.0 | 30.6 | 22.0 | 48.8 | +704.4% | 21.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 243.1 | 22.7 | 4337 | 216.4 | 153.9 | 4384 | +1684.0% | 26.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | c2_q2_p50_us | 80.9 | 9.68 | 512.7 | 34.7 | 25.5 | 48.2 | +533.6% | 16.9 | **N better** |
| N vs P+ | c4_q1_p50_us | 121.9 | 11.9 | 1405 | 67.0 | 48.1 | 210.4 | +1052.9% | 26.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 863.5 | 381.5 | 7610 | 499.9 | 444.7 | 8420 | +781.3% | 15.2 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c4_q2_p50_us | 156.7 | 16.6 | 1298 | 88.8 | 63.9 | 73.5 | +728.8% | 17.9 | **N better** |
| N vs P+ | c4_q2_p99_us | 1600 | 492.5 | 5036 | 2041 | 1485 | 4439 | +214.7% | 2.31 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 491.3 | 74.1 | 919.6 | 33.0 | 57.4 | 237.6 | +87.2% | 7.46 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p99_us | 1892 | 410.6 | 3132 | 375.8 | 393.6 | 2209 | +65.5% | 3.15 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1481 | 160.5 | 862.3 | 45.8 | 118.0 | 118.1 | -41.8% | 5.24 | **N better** |
| N vs P+ | pss_mib | 827.3 | 26.1 | 493.8 | 3.23 | 18.6 | 2.57 | -40.3% | 18.0 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 147727 | 27280 | 54970 | 13789 | 21614 | 357392 | -62.8% | 4.29 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 131.6 | 11.0 | 143.5 | 5.43 | 8.65 | 8.91 | +9.0% | 1.37 | no difference |
| N vs P | q1_p99_us | 958.0 | 480.0 | 626.1 | 149.5 | 355.5 | 204.2 | -34.7% | 0.93 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q2_p50_us | 156.2 | 9.98 | 216.9 | 11.8 | 10.9 | 6.69 | +38.9% | 5.57 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 606711 | 21094 | 185.3 | 16.9 | 14916 | 20506 | -100.0% | 40.7 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 65.6 | 5.70 | 63.9 | 5.57 | 5.64 | 48.7 | -2.7% | 0.31 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 243.1 | 22.7 | 1096 | 51.6 | 39.9 | 55.9 | +350.9% | 21.4 | **N better** |
| N vs P | c2_q2_p50_us | 80.9 | 9.68 | 192.4 | 33.6 | 24.7 | 35.9 | +137.7% | 4.51 | **N better** |
| N vs P | c4_q1_p50_us | 121.9 | 11.9 | 135.9 | 26.6 | 20.6 | 85.3 | +11.5% | 0.68 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 863.5 | 381.5 | 3450 | 642.1 | 528.1 | 448.7 | +299.5% | 4.90 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 156.7 | 16.6 | 355.8 | 69.4 | 50.5 | 18.6 | +127.1% | 3.95 | **N better** |
| N vs P | c4_q2_p99_us | 1600 | 492.5 | 8062 | 3566 | 2545 | 5735 | +403.9% | 2.54 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 491.3 | 74.1 | 1387614 | 16909 | 11957 | 10782 | +282313.7% | 116.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p99_us | 1892 | 410.6 | 1524983 | 26239 | 18556 | 27644 | +80510.9% | 82.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1481 | 160.5 | 0.72 | 0.00 | 113.5 | 27.8 | -100.0% | 13.0 | **N better** |
| N vs P | pss_mib | 827.3 | 26.1 | 487.8 | 0.69 | 18.4 | 2.38 | -41.0% | 18.4 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 147727 | 27280 | 53232 | 12921 | 21344 | 361765 | -64.0% | 4.43 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 507.6 | 18.4 | 143.5 | 5.43 | 13.6 | 96.8 | -71.7% | 26.8 | **P better** |
| P+ vs P | q1_p99_us | 2352 | 466.0 | 626.1 | 149.5 | 346.0 | 2069 | -73.4% | 4.99 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | q2_p50_us | 458.4 | 33.1 | 216.9 | 11.8 | 24.9 | 6.48 | -52.7% | 9.71 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 257.6 | 21.0 | 263.6 | 23.8 | 22.5 | 195.8 | +2.3% | 0.27 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q4_p50_us | 86656 | 1786 | 5843 | 66.3 | 1264 | 4035 | -93.3% | 63.9 | **P better** |
| P+ vs P | q5_p50_us | 930441 | 26551 | 185.3 | 16.9 | 18775 | 16535 | -100.0% | 49.5 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 528.0 | 30.6 | 63.9 | 5.57 | 22.0 | 3.35 | -87.9% | 21.1 | **P better** |
| P+ vs P | c2_q1_p99_us | 4337 | 216.4 | 1096 | 51.6 | 157.3 | 4385 | -74.7% | 20.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c2_q2_p50_us | 512.7 | 34.7 | 192.4 | 33.6 | 34.2 | 32.2 | -62.5% | 9.38 | **P better** |
| P+ vs P | c4_q1_p50_us | 1405 | 67.0 | 135.9 | 26.6 | 51.0 | 226.7 | -90.3% | 24.9 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 7610 | 499.9 | 3450 | 642.1 | 575.4 | 8410 | -54.7% | 7.23 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | c4_q2_p50_us | 1298 | 88.8 | 355.8 | 69.4 | 79.7 | 74.2 | -72.6% | 11.8 | **P better** |
| P+ vs P | c4_q2_p99_us | 5036 | 2041 | 8062 | 3566 | 2905 | 6763 | +60.1% | 1.04 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 919.6 | 33.0 | 1387614 | 16909 | 11957 | 10780 | +150797.1% | 116.0 | **P+ better** |
| P+ vs P | write_p99_us | 3132 | 375.8 | 1524983 | 26239 | 18556 | 27586 | +48596.6% | 82.0 | **P+ better** |
| P+ vs P | commits_per_s | 862.3 | 45.8 | 0.72 | 0.00 | 32.4 | 114.8 | -99.9% | 26.6 | **P+ better** |
| P+ vs P | pss_mib | 493.8 | 3.23 | 487.8 | 0.69 | 2.34 | 1.87 | -1.2% | 2.54 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 54970 | 13789 | 53232 | 12921 | 13362 | 189424 | -3.2% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 143.5 | 5.43 | 134.3 | 3.97 | 4.75 | 9.04 | -6.5% | 1.95 | no difference |
| P vs M | q1_p99_us | 626.1 | 149.5 | 829.8 | 304.7 | 240.0 | 526.9 | +32.5% | 0.85 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q2_p50_us | 216.9 | 11.8 | 214.2 | 13.9 | 12.9 | 6.98 | -1.2% | 0.21 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 263.6 | 23.8 | 256.2 | 26.7 | 25.3 | 230.0 | -2.8% | 0.29 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | q4_p50_us | 5843 | 66.3 | 5804 | 62.2 | 64.3 | 411.2 | -0.7% | 0.61 | BELOW FLOOR |
| P vs M | q5_p50_us | 185.3 | 16.9 | 190.3 | 11.9 | 14.6 | 70.7 | +2.7% | 0.34 | BELOW FLOOR |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 63.9 | 5.57 | 60.9 | 2.54 | 4.33 | 11.2 | -4.7% | 0.70 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1096 | 51.6 | 1079 | 56.3 | 54.0 | 308.0 | -1.5% | 0.31 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 192.4 | 33.6 | 183.2 | 20.4 | 27.8 | 33.3 | -4.8% | 0.33 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 135.9 | 26.6 | 157.0 | 21.4 | 24.1 | 84.9 | +15.6% | 0.88 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 3450 | 642.1 | 3323 | 535.0 | 591.0 | 1017 | -3.7% | 0.21 | BELOW FLOOR |
| P vs M | c4_q2_p50_us | 355.8 | 69.4 | 375.1 | 56.6 | 63.3 | 85.2 | +5.4% | 0.30 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 8062 | 3566 | 6745 | 2729 | 3175 | 5728 | -16.3% | 0.41 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 1387614 | 16909 | 1374004 | 30207 | 24479 | 13488 | -1.0% | 0.56 | no difference |
| P vs M | write_p99_us | 1524983 | 26239 | 1497826 | 32797 | 29700 | 42057 | -1.8% | 0.91 | BELOW FLOOR |
| P vs M | commits_per_s | 0.72 | 0.00 | 0.73 | 0.02 | 0.01 | 0.01 | +1.4% | 0.86 | BELOW FLOOR |
| P vs M | pss_mib | 487.8 | 0.69 | 488.7 | 1.29 | 1.04 | 1.16 | +0.2% | 0.81 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 53232 | 12921 | 55028 | 11583 | 12270 | 199261 | +3.4% | 0.15 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁵ accounts, seed 7

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 133.4 | 13.3 | 509.1 | 20.4 | 17.2 | 57.1 | +281.8% | 21.8 | **N better** |
| N vs P+ | q1_p99_us | 1244 | 846.6 | 2827 | 594.3 | 731.4 | 1581 | +127.3% | 2.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | q2_p50_us | 162.6 | 15.1 | 446.4 | 24.1 | 20.1 | 42.3 | +174.5% | 14.1 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 609338 | 14077 | 904738 | 30935 | 24033 | 27844 | +48.5% | 12.3 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 129.2 | 17.9 | 515.0 | 36.1 | 28.5 | 3.27 | +298.5% | 13.5 | **N better** |
| N vs P+ | c2_q1_p99_us | 614.3 | 357.9 | 4837 | 688.1 | 548.4 | 848.9 | +687.4% | 7.70 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 159.6 | 24.5 | 454.3 | 22.4 | 23.5 | 14.8 | +184.7% | 12.5 | **N better** |
| N vs P+ | c4_q1_p50_us | 159.4 | 25.7 | 1494 | 90.2 | 66.3 | 65.6 | +837.2% | 20.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1222 | 514.3 | 6971 | 741.8 | 638.3 | 1210 | +470.5% | 9.01 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 175.8 | 43.3 | 1362 | 91.3 | 71.5 | 234.7 | +674.7% | 16.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 2040 | 782.8 | 7075 | 1567 | 1238 | 981.0 | +246.8% | 4.07 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 616.1 | 31.6 | 989.9 | 57.8 | 46.6 | 116.0 | +60.7% | 8.02 | **N better** |
| N vs P+ | write_p99_us | 1614 | 646.0 | 3078 | 242.1 | 487.8 | 4752 | +90.7% | 3.00 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1398 | 136.2 | 850.0 | 47.4 | 102.0 | 262.5 | -39.2% | 5.37 | **N better** |
| N vs P+ | pss_mib | 804.5 | 16.8 | 493.8 | 0.37 | 11.9 | 38.1 | -38.6% | 26.1 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 143252 | 30335 | 10393 | 2537 | 21525 | 333815 | -92.7% | 6.17 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 133.4 | 13.3 | 140.9 | 5.47 | 10.2 | 13.5 | +5.6% | 0.74 | BELOW FLOOR |
| N vs P | q1_p99_us | 1244 | 846.6 | 679.8 | 183.0 | 612.5 | 1602 | -45.3% | 0.92 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | q2_p50_us | 162.6 | 15.1 | 225.5 | 11.6 | 13.4 | 19.0 | +38.7% | 4.68 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 609338 | 14077 | 188.4 | 13.4 | 9954 | 25928 | -100.0% | 61.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 129.2 | 17.9 | 59.1 | 3.00 | 12.8 | 3.19 | -54.3% | 5.48 | **P better** |
| N vs P | c2_q1_p99_us | 614.3 | 357.9 | 1011 | 153.3 | 275.3 | 700.0 | +64.5% | 1.44 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 159.6 | 24.5 | 155.3 | 10.4 | 18.8 | 72.7 | -2.7% | 0.23 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p50_us | 159.4 | 25.7 | 129.7 | 13.5 | 20.5 | 176.2 | -18.7% | 1.45 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1222 | 514.3 | 2745 | 400.0 | 460.7 | 456.0 | +124.7% | 3.31 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 175.8 | 43.3 | 296.5 | 28.8 | 36.8 | 187.8 | +68.7% | 3.28 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 2040 | 782.8 | 7490 | 4268 | 3068 | 1766 | +267.1% | 1.78 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 616.1 | 31.6 | 1349638 | 18133 | 12822 | 15624 | +218954.3% | 105.2 | **N better** |
| N vs P | write_p99_us | 1614 | 646.0 | 1488031 | 18168 | 12855 | 37650 | +92095.1% | 115.6 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1398 | 136.2 | 0.74 | 0.01 | 96.3 | 67.6 | -99.9% | 14.5 | **N better** |
| N vs P | pss_mib | 804.5 | 16.8 | 487.4 | 1.65 | 12.0 | 38.1 | -39.4% | 26.5 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 143252 | 30335 | 18388 | 3968 | 21633 | 335913 | -87.2% | 5.77 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 509.1 | 20.4 | 140.9 | 5.47 | 15.0 | 55.7 | -72.3% | 24.6 | **P better** |
| P+ vs P | q1_p99_us | 2827 | 594.3 | 679.8 | 183.0 | 439.7 | 279.5 | -76.0% | 4.88 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q2_p50_us | 446.4 | 24.1 | 225.5 | 11.6 | 18.9 | 40.1 | -49.5% | 11.7 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 270.3 | 33.8 | 243.0 | 10.5 | 25.0 | 72.0 | -10.1% | 1.09 | BELOW FLOOR |
| P+ vs P | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q5_p50_us | 904738 | 30935 | 188.4 | 13.4 | 21875 | 10152 | -100.0% | 41.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 515.0 | 36.1 | 59.1 | 3.00 | 25.6 | 2.44 | -88.5% | 17.8 | **P better** |
| P+ vs P | c2_q1_p99_us | 4837 | 688.1 | 1011 | 153.3 | 498.5 | 536.8 | -79.1% | 7.68 | **P better** |
| P+ vs P | c2_q2_p50_us | 454.3 | 22.4 | 155.3 | 10.4 | 17.5 | 73.6 | -65.8% | 17.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p50_us | 1494 | 90.2 | 129.7 | 13.5 | 64.5 | 164.3 | -91.3% | 21.1 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 6971 | 741.8 | 2745 | 400.0 | 595.9 | 1151 | -60.6% | 7.09 | **P better** |
| P+ vs P | c4_q2_p50_us | 1362 | 91.3 | 296.5 | 28.8 | 67.7 | 271.8 | -78.2% | 15.7 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 7075 | 1567 | 7490 | 4268 | 3215 | 1904 | +5.9% | 0.13 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 989.9 | 57.8 | 1349638 | 18133 | 12822 | 15623 | +136234.8% | 105.2 | **P+ better** |
| P+ vs P | write_p99_us | 3078 | 242.1 | 1488031 | 18168 | 12848 | 37480 | +48247.8% | 115.6 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 850.0 | 47.4 | 0.74 | 0.01 | 33.5 | 253.7 | -99.9% | 25.4 | **P+ better** |
| P+ vs P | pss_mib | 493.8 | 0.37 | 487.4 | 1.65 | 1.20 | 0.52 | -1.3% | 5.36 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 10393 | 2537 | 18388 | 3968 | 3330 | 46013 | +76.9% | 2.40 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 140.9 | 5.47 | 139.9 | 1.36 | 3.99 | 4.92 | -0.7% | 0.25 | BELOW FLOOR |
| P vs M | q1_p99_us | 679.8 | 183.0 | 975.4 | 458.1 | 348.8 | 271.5 | +43.5% | 0.85 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q2_p50_us | 225.5 | 11.6 | 235.6 | 18.0 | 15.2 | 12.7 | +4.5% | 0.66 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 243.0 | 10.5 | 247.8 | 16.3 | 13.7 | 99.1 | +2.0% | 0.35 | BELOW FLOOR |
| P vs M | q4_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q5_p50_us | 188.4 | 13.4 | 185.4 | 5.47 | 10.3 | 78.4 | -1.6% | 0.30 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 59.1 | 3.00 | 60.2 | 2.65 | 2.83 | 1.73 | +1.9% | 0.40 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1011 | 153.3 | 990.7 | 107.7 | 132.5 | 242.9 | -2.0% | 0.15 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 155.3 | 10.4 | 163.0 | 19.3 | 15.5 | 88.5 | +4.9% | 0.49 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p50_us | 129.7 | 13.5 | 158.6 | 30.5 | 23.5 | 183.9 | +22.3% | 1.23 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2745 | 400.0 | 3166 | 655.2 | 542.8 | 195.8 | +15.3% | 0.78 | no difference |
| P vs M | c4_q2_p50_us | 296.5 | 28.8 | 336.1 | 39.6 | 34.6 | 192.7 | +13.3% | 1.14 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q2_p99_us | 7490 | 4268 | 9362 | 4277 | 4272 | 2668 | +25.0% | 0.44 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 1349638 | 18133 | 1362051 | 30545 | 25117 | 23585 | +0.9% | 0.49 | BELOW FLOOR |
| P vs M | write_p99_us | 1488031 | 18168 | 1445370 | 31589 | 25768 | 52998 | -2.9% | 1.66 | BELOW FLOOR |
| P vs M | commits_per_s | 0.74 | 0.01 | 0.74 | 0.01 | 0.01 | 0.02 | -0.7% | 0.43 | BELOW FLOOR |
| P vs M | pss_mib | 487.4 | 1.65 | 485.5 | 4.09 | 3.12 | 2.08 | -0.4% | 0.61 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 18388 | 3968 | 52668 | 19291 | 13927 | 102363 | +186.4% | 2.46 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁵ accounts, seed 42

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 137.2 | 9.90 | 512.9 | 8.50 | 9.23 | 20.4 | +273.7% | 40.7 | **N better** |
| N vs P+ | q1_p99_us | 1078 | 493.1 | 2982 | 590.4 | 543.9 | 843.7 | +176.8% | 3.50 | **N better** |
| N vs P+ | q2_p50_us | 148.8 | 8.82 | 446.1 | 13.3 | 11.3 | 6.98 | +199.9% | 26.3 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 612075 | 32014 | 922351 | 25440 | 28914 | 47274 | +50.7% | 10.7 | **N better** |
| N vs P+ | q6_p50_us | 1146414 | 53050 | 965862 | 34219 | 44639 | 126647 | -15.7% | 4.04 | **P+ better** |
| N vs P+ | c2_q1_p50_us | 72.1 | 14.0 | 492.5 | 14.3 | 14.1 | 47.7 | +583.4% | 29.7 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 267.9 | 107.0 | 4457 | 242.9 | 187.7 | 2495 | +1563.7% | 22.3 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | c2_q2_p50_us | 89.4 | 18.9 | 470.4 | 20.5 | 19.8 | 41.8 | +426.0% | 19.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 124.2 | 23.5 | 1491 | 78.1 | 57.6 | 83.7 | +1101.0% | 23.7 | **N better** |
| N vs P+ | c4_q1_p99_us | 1159 | 576.2 | 8560 | 764.6 | 677.0 | 1521 | +638.8% | 10.9 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 160.5 | 25.7 | 1353 | 57.2 | 44.3 | 74.9 | +743.3% | 26.9 | **N better** |
| N vs P+ | c4_q2_p99_us | 4061 | 3094 | 8345 | 1418 | 2407 | 7945 | +105.5% | 1.78 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | write_p50_us | 617.4 | 72.6 | 915.5 | 38.7 | 58.2 | 179.0 | +48.3% | 5.12 | **N better** |
| N vs P+ | write_p99_us | 1861 | 331.0 | 2822 | 77.8 | 240.4 | 721.9 | +51.7% | 4.00 | **N better** |
| N vs P+ | commits_per_s | 1342 | 103.9 | 837.2 | 50.9 | 81.8 | 286.3 | -37.6% | 6.17 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 830.4 | 14.2 | 488.9 | 6.17 | 11.0 | 39.2 | -41.1% | 31.2 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 153351 | 37630 | 54642 | 12835 | 28114 | 365511 | -64.4% | 3.51 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 137.2 | 9.90 | 140.4 | 2.33 | 7.19 | 9.51 | +2.3% | 0.44 | BELOW FLOOR |
| N vs P | q1_p99_us | 1078 | 493.1 | 803.6 | 246.2 | 389.7 | 208.8 | -25.4% | 0.70 | no difference |
| N vs P | q2_p50_us | 148.8 | 8.82 | 224.4 | 8.60 | 8.71 | 8.66 | +50.8% | 8.68 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 612075 | 32014 | 182.1 | 8.97 | 22638 | 39969 | -100.0% | 27.0 | **P better** |
| N vs P | q6_p50_us | 1146414 | 53050 | 29960 | 789.4 | 37516 | 123250 | -97.4% | 29.8 | **P better** |
| N vs P | c2_q1_p50_us | 72.1 | 14.0 | 62.3 | 4.11 | 10.3 | 35.6 | -13.5% | 0.95 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 267.9 | 107.0 | 1120 | 97.8 | 102.5 | 467.0 | +318.0% | 8.31 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c2_q2_p50_us | 89.4 | 18.9 | 166.5 | 18.2 | 18.6 | 44.4 | +86.2% | 4.15 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 124.2 | 23.5 | 147.0 | 13.1 | 19.0 | 18.1 | +18.4% | 1.20 | no difference |
| N vs P | c4_q1_p99_us | 1159 | 576.2 | 3199 | 707.0 | 644.9 | 1111 | +176.1% | 3.16 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q2_p50_us | 160.5 | 25.7 | 294.6 | 31.3 | 28.7 | 51.2 | +83.6% | 4.68 | **N better** |
| N vs P | c4_q2_p99_us | 4061 | 3094 | 15590 | 10891 | 8006 | 10318 | +283.9% | 1.44 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | write_p50_us | 617.4 | 72.6 | 1357341 | 26883 | 19009 | 32265 | +219735.7% | 71.4 | **N better** |
| N vs P | write_p99_us | 1861 | 331.0 | 1507126 | 29026 | 20526 | 91591 | +80889.6% | 73.3 | **N better** |
| N vs P | commits_per_s | 1342 | 103.9 | 0.73 | 0.01 | 73.5 | 278.9 | -99.9% | 18.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_mib | 830.4 | 14.2 | 478.9 | 5.86 | 10.9 | 39.1 | -42.3% | 32.3 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 153351 | 37630 | 49581 | 13690 | 28315 | 370507 | -67.7% | 3.66 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 512.9 | 8.50 | 140.4 | 2.33 | 6.24 | 18.7 | -72.6% | 59.7 | **P better** |
| P+ vs P | q1_p99_us | 2982 | 590.4 | 803.6 | 246.2 | 452.3 | 821.6 | -73.1% | 4.82 | **P better** |
| P+ vs P | q2_p50_us | 446.1 | 13.3 | 224.4 | 8.60 | 11.2 | 5.41 | -49.7% | 19.8 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 234.4 | 14.2 | 235.9 | 14.7 | 14.4 | 46.0 | +0.7% | 0.11 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 85614 | 1396 | 5819 | 210.4 | 998.3 | 5850 | -93.2% | 79.9 | **P better** |
| P+ vs P | q5_p50_us | 922351 | 25440 | 182.1 | 8.97 | 17988 | 25246 | -100.0% | 51.3 | **P better** |
| P+ vs P | q6_p50_us | 965862 | 34219 | 29960 | 789.4 | 24203 | 29143 | -96.9% | 38.7 | **P better** |
| P+ vs P | c2_q1_p50_us | 492.5 | 14.3 | 62.3 | 4.11 | 10.5 | 31.8 | -87.3% | 40.9 | **P better** |
| P+ vs P | c2_q1_p99_us | 4457 | 242.9 | 1120 | 97.8 | 185.1 | 2513 | -74.9% | 18.0 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | c2_q2_p50_us | 470.4 | 20.5 | 166.5 | 18.2 | 19.4 | 28.8 | -64.6% | 15.7 | **P better** |
| P+ vs P | c4_q1_p50_us | 1491 | 78.1 | 147.0 | 13.1 | 56.0 | 83.0 | -90.1% | 24.0 | **P better** |
| P+ vs P | c4_q1_p99_us | 8560 | 764.6 | 3199 | 707.0 | 736.3 | 1040 | -62.6% | 7.28 | **P better** |
| P+ vs P | c4_q2_p50_us | 1353 | 57.2 | 294.6 | 31.3 | 46.1 | 88.6 | -78.2% | 23.0 | **P better** |
| P+ vs P | c4_q2_p99_us | 8345 | 1418 | 15590 | 10891 | 7766 | 13005 | +86.8% | 0.93 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P+ vs P | write_p50_us | 915.5 | 38.7 | 1357341 | 26883 | 19009 | 32264 | +148164.6% | 71.4 | **P+ better** |
| P+ vs P | write_p99_us | 2822 | 77.8 | 1507126 | 29026 | 20525 | 91594 | +53302.1% | 73.3 | **P+ better** |
| P+ vs P | commits_per_s | 837.2 | 50.9 | 0.73 | 0.01 | 36.0 | 64.9 | -99.9% | 23.2 | **P+ better** |
| P+ vs P | pss_mib | 488.9 | 6.17 | 478.9 | 5.86 | 6.01 | 2.86 | -2.0% | 1.66 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 54642 | 12835 | 49581 | 13690 | 13270 | 192225 | -9.3% | 0.38 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 140.4 | 2.33 | 141.3 | 5.57 | 4.27 | 4.87 | +0.7% | 0.23 | BELOW FLOOR |
| P vs M | q1_p99_us | 803.6 | 246.2 | 886.7 | 229.9 | 238.2 | 95.0 | +10.3% | 0.35 | BELOW FLOOR |
| P vs M | q2_p50_us | 224.4 | 8.60 | 213.9 | 4.72 | 6.94 | 11.2 | -4.7% | 1.51 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 235.9 | 14.7 | 247.4 | 13.8 | 14.3 | 11.9 | +4.9% | 0.81 | BELOW FLOOR |
| P vs M | q4_p50_us | 5819 | 210.4 | 5792 | 89.3 | 161.6 | 76.1 | -0.5% | 0.16 | BELOW FLOOR |
| P vs M | q5_p50_us | 182.1 | 8.97 | 178.6 | 4.93 | 7.24 | 32.8 | -1.9% | 0.49 | BELOW FLOOR |
| P vs M | q6_p50_us | 29960 | 789.4 | 30917 | 1019 | 911.2 | 792.5 | +3.2% | 1.05 | no difference |
| P vs M | c2_q1_p50_us | 62.3 | 4.11 | 60.1 | 1.66 | 3.13 | 5.28 | -3.5% | 0.71 | BELOW FLOOR |
| P vs M | c2_q1_p99_us | 1120 | 97.8 | 1040 | 125.2 | 112.3 | 396.9 | -7.2% | 0.71 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c2_q2_p50_us | 166.5 | 18.2 | 166.8 | 20.6 | 19.4 | 23.7 | +0.2% | 0.01 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 147.0 | 13.1 | 137.9 | 29.0 | 22.5 | 76.4 | -6.2% | 0.40 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p99_us | 3199 | 707.0 | 2836 | 386.1 | 569.6 | 314.0 | -11.4% | 0.64 | no difference |
| P vs M | c4_q2_p50_us | 294.6 | 31.3 | 325.5 | 32.7 | 32.0 | 91.3 | +10.5% | 0.97 | BELOW FLOOR |
| P vs M | c4_q2_p99_us | 15590 | 10891 | 17583 | 9026 | 10002 | 10384 | +12.8% | 0.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | write_p50_us | 1357341 | 26883 | 1360861 | 16358 | 22252 | 50626 | +0.3% | 0.16 | BELOW FLOOR |
| P vs M | write_p99_us | 1507126 | 29026 | 1511088 | 36614 | 33039 | 110259 | +0.3% | 0.12 | BELOW FLOOR |
| P vs M | commits_per_s | 0.73 | 0.01 | 0.73 | 0.01 | 0.01 | 0.03 | +0.5% | 0.33 | BELOW FLOOR |
| P vs M | pss_mib | 478.9 | 5.86 | 485.4 | 1.88 | 4.35 | 0.85 | +1.3% | 1.48 | no difference |
| P vs M | pss_growth_bytes_per_key_read | 49581 | 13690 | 43178 | 8740 | 11485 | 203176 | -12.9% | 0.56 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁵ accounts, seed 100

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 136.1 | 6.23 | 527.9 | 21.0 | 15.5 | 30.4 | +288.0% | 25.3 | **N better** |
| N vs P+ | q1_p99_us | 970.5 | 600.8 | 2814 | 417.9 | 517.5 | 3254 | +190.0% | 3.56 | REFUSED (warm-up MAD above 15% of the median on P+) |
| N vs P+ | q2_p50_us | 159.8 | 18.5 | 462.5 | 24.0 | 21.4 | 46.3 | +189.4% | 14.1 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 620991 | 17907 | 922961 | 35803 | 28306 | 36528 | +48.6% | 10.7 | **N better** |
| N vs P+ | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | c2_q1_p50_us | 91.0 | 23.4 | 505.9 | 22.7 | 23.1 | 58.8 | +455.7% | 18.0 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 388.5 | 156.7 | 4600 | 286.1 | 230.7 | 511.7 | +1084.0% | 18.3 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q2_p50_us | 108.1 | 37.3 | 460.5 | 11.5 | 27.6 | 67.8 | +325.9% | 12.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 140.3 | 36.7 | 1417 | 71.8 | 57.0 | 103.5 | +910.2% | 22.4 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p99_us | 1018 | 330.9 | 7303 | 1058 | 783.7 | 1197 | +617.6% | 8.02 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 167.3 | 19.3 | 1379 | 70.3 | 51.5 | 88.1 | +724.3% | 23.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p99_us | 1818 | 745.6 | 6889 | 1739 | 1338 | 3542 | +279.1% | 3.79 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | write_p50_us | 654.2 | 47.4 | 982.5 | 38.1 | 43.0 | 306.8 | +50.2% | 7.63 | **N better** |
| N vs P+ | write_p99_us | 1890 | 500.9 | 3362 | 352.3 | 433.0 | 2673 | +77.8% | 3.40 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | commits_per_s | 1384 | 181.9 | 816.3 | 30.4 | 130.4 | 325.7 | -41.0% | 4.36 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | pss_mib | 858.5 | 10.4 | 496.3 | 0.29 | 7.38 | 3.85 | -42.2% | 49.1 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 156367 | 33572 | 55162 | 13498 | 25586 | 343323 | -64.7% | 3.96 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 136.1 | 6.23 | 139.0 | 3.39 | 5.02 | 32.8 | +2.2% | 0.58 | BELOW FLOOR |
| N vs P | q1_p99_us | 970.5 | 600.8 | 711.2 | 142.8 | 436.6 | 86.0 | -26.7% | 0.59 | no difference |
| N vs P | q2_p50_us | 159.8 | 18.5 | 243.2 | 14.0 | 16.4 | 7.13 | +52.2% | 5.08 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 620991 | 17907 | 218.3 | 32.0 | 12662 | 35787 | -100.0% | 49.0 | **P better** |
| N vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | c2_q1_p50_us | 91.0 | 23.4 | 61.8 | 2.35 | 16.6 | 54.6 | -32.1% | 1.76 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 388.5 | 156.7 | 1114 | 88.9 | 127.4 | 537.8 | +186.8% | 5.69 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q2_p50_us | 108.1 | 37.3 | 169.5 | 9.01 | 27.2 | 50.4 | +56.8% | 2.26 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 140.3 | 36.7 | 141.9 | 27.7 | 32.5 | 113.4 | +1.2% | 0.05 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q1_p99_us | 1018 | 330.9 | 2742 | 602.8 | 486.3 | 1389 | +169.5% | 3.55 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p50_us | 167.3 | 19.3 | 346.4 | 53.5 | 40.2 | 186.6 | +107.0% | 4.45 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p99_us | 1818 | 745.6 | 9023 | 4562 | 3268 | 740.5 | +396.5% | 2.20 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | write_p50_us | 654.2 | 47.4 | 1385662 | 41845 | 29589 | 17091 | +211695.1% | 46.8 | **N better** |
| N vs P | write_p99_us | 1890 | 500.9 | 1518511 | 40112 | 28366 | 44715 | +80223.8% | 53.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1384 | 181.9 | 0.72 | 0.01 | 128.6 | 319.7 | -99.9% | 10.8 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | pss_mib | 858.5 | 10.4 | 488.2 | 2.31 | 7.55 | 1.78 | -43.1% | 49.0 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 156367 | 33572 | 52912 | 14335 | 25812 | 338439 | -66.2% | 4.01 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 527.9 | 21.0 | 139.0 | 3.39 | 15.1 | 13.0 | -73.7% | 25.8 | **P better** |
| P+ vs P | q1_p99_us | 2814 | 417.9 | 711.2 | 142.8 | 312.2 | 3254 | -74.7% | 6.74 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | q2_p50_us | 462.5 | 24.0 | 243.2 | 14.0 | 19.6 | 45.8 | -47.4% | 11.2 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 268.3 | 12.7 | 230.7 | 12.4 | 12.5 | 21.5 | -14.0% | 3.00 | **P better** |
| P+ vs P | q4_p50_us | 85507 | 1239 | 5915 | 144.5 | 881.8 | 4079 | -93.1% | 90.3 | **P better** |
| P+ vs P | q5_p50_us | 922961 | 35803 | 218.3 | 32.0 | 25317 | 7321 | -100.0% | 36.4 | **P better** |
| P+ vs P | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | c2_q1_p50_us | 505.9 | 22.7 | 61.8 | 2.35 | 16.2 | 25.3 | -87.8% | 27.5 | **P better** |
| P+ vs P | c2_q1_p99_us | 4600 | 286.1 | 1114 | 88.9 | 211.8 | 172.0 | -75.8% | 16.5 | **P better** |
| P+ vs P | c2_q2_p50_us | 460.5 | 11.5 | 169.5 | 9.01 | 10.3 | 45.4 | -63.2% | 28.2 | **P better** |
| P+ vs P | c4_q1_p50_us | 1417 | 71.8 | 141.9 | 27.7 | 54.4 | 53.2 | -90.0% | 23.4 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 7303 | 1058 | 2742 | 602.8 | 860.8 | 1524 | -62.4% | 5.30 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1379 | 70.3 | 346.4 | 53.5 | 62.5 | 185.6 | -74.9% | 16.5 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 6889 | 1739 | 9023 | 4562 | 3452 | 3518 | +31.0% | 0.62 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | write_p50_us | 982.5 | 38.1 | 1385662 | 41845 | 29589 | 17089 | +140932.3% | 46.8 | **P+ better** |
| P+ vs P | write_p99_us | 3362 | 352.3 | 1518511 | 40112 | 28365 | 44641 | +45069.4% | 53.4 | **P+ better** |
| P+ vs P | commits_per_s | 816.3 | 30.4 | 0.72 | 0.01 | 21.5 | 62.1 | -99.9% | 37.9 | **P+ better** |
| P+ vs P | pss_mib | 496.3 | 0.29 | 488.2 | 2.31 | 1.65 | 4.22 | -1.6% | 4.96 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 55162 | 13498 | 52912 | 14335 | 13922 | 189805 | -4.1% | 0.16 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 139.0 | 3.39 | 141.0 | 2.55 | 3.00 | 20.3 | +1.4% | 0.67 | BELOW FLOOR |
| P vs M | q1_p99_us | 711.2 | 142.8 | 744.2 | 224.6 | 188.2 | 77.4 | +4.6% | 0.18 | BELOW FLOOR |
| P vs M | q2_p50_us | 243.2 | 14.0 | 229.3 | 16.0 | 15.0 | 43.6 | -5.7% | 0.93 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 230.7 | 12.4 | 242.6 | 19.6 | 16.4 | 22.9 | +5.1% | 0.72 | BELOW FLOOR |
| P vs M | q4_p50_us | 5915 | 144.5 | 5829 | 95.0 | 122.3 | 203.5 | -1.4% | 0.70 | BELOW FLOOR |
| P vs M | q5_p50_us | 218.3 | 32.0 | 196.9 | 19.1 | 26.3 | 102.8 | -9.8% | 0.81 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | q6_p50_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | c2_q1_p50_us | 61.8 | 2.35 | 59.9 | 3.25 | 2.84 | 56.8 | -3.1% | 0.68 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1114 | 88.9 | 1122 | 104.0 | 96.8 | 679.3 | +0.7% | 0.08 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q2_p50_us | 169.5 | 9.01 | 160.5 | 11.1 | 10.1 | 125.8 | -5.3% | 0.89 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c4_q1_p50_us | 141.9 | 27.7 | 155.1 | 13.4 | 21.8 | 74.2 | +9.3% | 0.61 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | c4_q1_p99_us | 2742 | 602.8 | 3316 | 307.4 | 478.5 | 1243 | +20.9% | 1.20 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 346.4 | 53.5 | 314.6 | 26.6 | 42.3 | 194.7 | -9.2% | 0.75 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 9023 | 4562 | 11311 | 5424 | 5011 | 1785 | +25.4% | 0.46 | no difference |
| P vs M | write_p50_us | 1385662 | 41845 | 1390151 | 13856 | 31169 | 28795 | +0.3% | 0.14 | BELOW FLOOR |
| P vs M | write_p99_us | 1518511 | 40112 | 1500105 | 18284 | 31171 | 56371 | -1.2% | 0.59 | BELOW FLOOR |
| P vs M | commits_per_s | 0.72 | 0.01 | 0.72 | 0.01 | 0.01 | 0.02 | -0.8% | 0.42 | BELOW FLOOR |
| P vs M | pss_mib | 488.2 | 2.31 | 484.0 | 6.91 | 5.15 | 3.87 | -0.8% | 0.80 | no difference |
| P vs M | pss_growth_bytes_per_key_read | 52912 | 14335 | 43477 | 8940 | 11946 | 191140 | -17.8% | 0.79 | REFUSED (warm-up MAD above 15% of the median on P and M) |

## `single`, 10⁵ accounts, seed 2024

| pair | metric | med A | MAD A | med B | MAD B | pooled | floor | rel | MADs | verdict |
|---|---|---|---|---|---|---|---|---|---|---|
| N vs P+ | q1_p50_us | 129.3 | 8.12 | 520.6 | 8.33 | 8.23 | 32.1 | +302.6% | 47.6 | **N better** |
| N vs P+ | q1_p99_us | 1031 | 440.1 | 2753 | 348.1 | 396.8 | 945.6 | +167.2% | 4.34 | **N better** |
| N vs P+ | q2_p50_us | 151.7 | 8.54 | 445.3 | 12.3 | 10.6 | 7.02 | +193.6% | 27.7 | **N better** |
| N vs P+ | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P+ | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P+ | q5_p50_us | 650079 | 25127 | 956434 | 16633 | 21307 | 84385 | +47.1% | 14.4 | **N better** |
| N vs P+ | q6_p50_us | 1127777 | 67379 | 982628 | 24415 | 50675 | 197924 | -12.9% | 2.86 | BELOW FLOOR |
| N vs P+ | c2_q1_p50_us | 73.2 | 13.7 | 545.0 | 29.5 | 23.0 | 59.3 | +644.4% | 20.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c2_q1_p99_us | 583.1 | 358.3 | 4625 | 1889 | 1360 | 1699 | +693.1% | 2.97 | no difference |
| N vs P+ | c2_q2_p50_us | 97.3 | 19.3 | 493.1 | 11.2 | 15.8 | 63.4 | +406.8% | 25.1 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q1_p50_us | 122.8 | 6.17 | 1495 | 56.4 | 40.1 | 272.1 | +1118.2% | 34.2 | **N better** |
| N vs P+ | c4_q1_p99_us | 1550 | 314.5 | 7243 | 940.5 | 701.2 | 1380 | +367.2% | 8.12 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | c4_q2_p50_us | 140.9 | 21.7 | 1407 | 81.2 | 59.5 | 27.9 | +898.8% | 21.3 | **N better** |
| N vs P+ | c4_q2_p99_us | 3304 | 2033 | 6463 | 1650 | 1852 | 2594 | +95.6% | 1.71 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P+ | write_p50_us | 599.9 | 74.6 | 943.2 | 40.4 | 60.0 | 200.6 | +57.2% | 5.72 | **N better** |
| N vs P+ | write_p99_us | 2225 | 576.9 | 3453 | 684.1 | 632.7 | 4172 | +55.2% | 1.94 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P+ | commits_per_s | 1348 | 114.4 | 828.8 | 41.8 | 86.1 | 25.9 | -38.5% | 6.03 | **N better** |
| N vs P+ | pss_mib | 830.9 | 0.90 | 493.1 | 5.45 | 3.90 | 1.81 | -40.7% | 86.5 | **P+ better** |
| N vs P+ | pss_growth_bytes_per_key_read | 151863 | 35877 | 55579 | 13322 | 27061 | 403551 | -63.4% | 3.56 | REFUSED (warm-up MAD above 15% of the median on N and P+) |
| N vs P | q1_p50_us | 129.3 | 8.12 | 139.9 | 3.77 | 6.33 | 26.2 | +8.2% | 1.68 | BELOW FLOOR |
| N vs P | q1_p99_us | 1031 | 440.1 | 677.9 | 155.1 | 329.9 | 118.7 | -34.2% | 1.07 | no difference |
| N vs P | q2_p50_us | 151.7 | 8.54 | 225.9 | 12.9 | 11.0 | 2.25 | +48.9% | 6.77 | **N better** |
| N vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| N vs P | q3_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q4_p50_us | — | — | — | — | — | — | — | — | NOT RUN |
| N vs P | q5_p50_us | 650079 | 25127 | 190.5 | 15.1 | 17767 | 83482 | -100.0% | 36.6 | **P better** |
| N vs P | q6_p50_us | 1127777 | 67379 | 28703 | 331.3 | 47645 | 194460 | -97.5% | 23.1 | **P better** |
| N vs P | c2_q1_p50_us | 73.2 | 13.7 | 64.9 | 7.93 | 11.2 | 42.8 | -11.4% | 0.74 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c2_q1_p99_us | 583.1 | 358.3 | 1077 | 62.8 | 257.2 | 70.8 | +84.6% | 1.92 | no difference |
| N vs P | c2_q2_p50_us | 97.3 | 19.3 | 183.3 | 18.8 | 19.0 | 63.3 | +88.4% | 4.52 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | c4_q1_p50_us | 122.8 | 6.17 | 151.0 | 23.2 | 17.0 | 71.4 | +23.0% | 1.66 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q1_p99_us | 1550 | 314.5 | 3413 | 511.7 | 424.7 | 1159 | +120.2% | 4.39 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | c4_q2_p50_us | 140.9 | 21.7 | 309.5 | 24.8 | 23.3 | 132.3 | +119.7% | 7.23 | REFUSED (warm-up MAD above 15% of the median on P) |
| N vs P | c4_q2_p99_us | 3304 | 2033 | 15022 | 4383 | 3417 | 3334 | +354.7% | 3.43 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| N vs P | write_p50_us | 599.9 | 74.6 | 1412011 | 32879 | 23249 | 113040 | +235273.6% | 60.7 | **N better** |
| N vs P | write_p99_us | 2225 | 576.9 | 1536548 | 51008 | 36070 | 139520 | +68952.6% | 42.5 | REFUSED (warm-up MAD above 15% of the median on N) |
| N vs P | commits_per_s | 1348 | 114.4 | 0.71 | 0.02 | 80.9 | 18.0 | -99.9% | 16.7 | **N better** |
| N vs P | pss_mib | 830.9 | 0.90 | 488.2 | 1.28 | 1.11 | 3.58 | -41.2% | 309.0 | **P better** |
| N vs P | pss_growth_bytes_per_key_read | 151863 | 35877 | 54869 | 11228 | 26582 | 405841 | -63.9% | 3.65 | REFUSED (warm-up MAD above 15% of the median on N and P) |
| P+ vs P | q1_p50_us | 520.6 | 8.33 | 139.9 | 3.77 | 6.47 | 23.6 | -73.1% | 58.9 | **P better** |
| P+ vs P | q1_p99_us | 2753 | 348.1 | 677.9 | 155.1 | 269.5 | 943.8 | -75.4% | 7.70 | **P better** |
| P+ vs P | q2_p50_us | 445.3 | 12.3 | 225.9 | 12.9 | 12.6 | 6.84 | -49.3% | 17.4 | **P better** |
| P+ vs P | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P+ vs P | q3_p50_us | 269.3 | 28.4 | 275.7 | 39.0 | 34.1 | 51.3 | +2.3% | 0.19 | BELOW FLOOR |
| P+ vs P | q4_p50_us | 85807 | 1120 | 5839 | 30.9 | 792.5 | 2137 | -93.2% | 100.9 | **P better** |
| P+ vs P | q5_p50_us | 956434 | 16633 | 190.5 | 15.1 | 11761 | 12310 | -100.0% | 81.3 | **P better** |
| P+ vs P | q6_p50_us | 982628 | 24415 | 28703 | 331.3 | 17265 | 37633 | -97.1% | 55.3 | **P better** |
| P+ vs P | c2_q1_p50_us | 545.0 | 29.5 | 64.9 | 7.93 | 21.6 | 41.4 | -88.1% | 22.2 | **P better** |
| P+ vs P | c2_q1_p99_us | 4625 | 1889 | 1077 | 62.8 | 1337 | 1700 | -76.7% | 2.65 | no difference |
| P+ vs P | c2_q2_p50_us | 493.1 | 11.2 | 183.3 | 18.8 | 15.5 | 13.0 | -62.8% | 20.0 | **P better** |
| P+ vs P | c4_q1_p50_us | 1495 | 56.4 | 151.0 | 23.2 | 43.1 | 281.0 | -89.9% | 31.2 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q1_p99_us | 7243 | 940.5 | 3413 | 511.7 | 757.1 | 1770 | -52.9% | 5.06 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p50_us | 1407 | 81.2 | 309.5 | 24.8 | 60.1 | 132.2 | -78.0% | 18.3 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | c4_q2_p99_us | 6463 | 1650 | 15022 | 4383 | 3312 | 3037 | +132.4% | 2.58 | REFUSED (warm-up MAD above 15% of the median on P) |
| P+ vs P | write_p50_us | 943.2 | 40.4 | 1412011 | 32879 | 23249 | 113040 | +149612.2% | 60.7 | **P+ better** |
| P+ vs P | write_p99_us | 3453 | 684.1 | 1536548 | 51008 | 36071 | 139477 | +44399.3% | 42.5 | REFUSED (warm-up MAD above 15% of the median on P+) |
| P+ vs P | commits_per_s | 828.8 | 41.8 | 0.71 | 0.02 | 29.6 | 18.6 | -99.9% | 28.0 | **P+ better** |
| P+ vs P | pss_mib | 493.1 | 5.45 | 488.2 | 1.28 | 3.96 | 4.01 | -1.0% | 1.24 | no difference |
| P+ vs P | pss_growth_bytes_per_key_read | 55579 | 13322 | 54869 | 11228 | 12320 | 185568 | -1.3% | 0.06 | REFUSED (warm-up MAD above 15% of the median on P+ and P) |
| P vs M | q1_p50_us | 139.9 | 3.77 | 136.0 | 5.19 | 4.53 | 10.5 | -2.8% | 0.87 | BELOW FLOOR |
| P vs M | q1_p99_us | 677.9 | 155.1 | 884.1 | 245.0 | 205.0 | 218.7 | +30.4% | 1.01 | BELOW FLOOR |
| P vs M | q2_p50_us | 225.9 | 12.9 | 225.8 | 9.46 | 11.3 | 24.4 | -0.0% | 0.01 | BELOW FLOOR |
| P vs M | q2_p99_us | — | — | — | — | — | — | — | — | REFUSED (a run lacked this metric) |
| P vs M | q3_p50_us | 275.7 | 39.0 | 250.0 | 29.2 | 34.5 | 49.9 | -9.3% | 0.74 | BELOW FLOOR |
| P vs M | q4_p50_us | 5839 | 30.9 | 5896 | 140.1 | 101.5 | 108.7 | +1.0% | 0.56 | BELOW FLOOR |
| P vs M | q5_p50_us | 190.5 | 15.1 | 186.5 | 9.26 | 12.5 | 12.6 | -2.1% | 0.32 | BELOW FLOOR |
| P vs M | q6_p50_us | 28703 | 331.3 | 29549 | 795.9 | 609.6 | 5410 | +3.0% | 1.39 | BELOW FLOOR |
| P vs M | c2_q1_p50_us | 64.9 | 7.93 | 59.8 | 2.35 | 5.85 | 82.2 | -7.9% | 0.88 | REFUSED (warm-up MAD above 15% of the median on M) |
| P vs M | c2_q1_p99_us | 1077 | 62.8 | 1102 | 191.2 | 142.3 | 193.2 | +2.3% | 0.17 | BELOW FLOOR |
| P vs M | c2_q2_p50_us | 183.3 | 18.8 | 180.9 | 16.0 | 17.5 | 12.8 | -1.3% | 0.14 | BELOW FLOOR |
| P vs M | c4_q1_p50_us | 151.0 | 23.2 | 142.1 | 24.4 | 23.8 | 71.0 | -5.9% | 0.37 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q1_p99_us | 3413 | 511.7 | 3532 | 452.9 | 483.2 | 1310 | +3.5% | 0.25 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p50_us | 309.5 | 24.8 | 308.7 | 30.5 | 27.8 | 131.9 | -0.3% | 0.03 | REFUSED (warm-up MAD above 15% of the median on P) |
| P vs M | c4_q2_p99_us | 15022 | 4383 | 18761 | 3895 | 4146 | 9510 | +24.9% | 0.90 | REFUSED (warm-up MAD above 15% of the median on P and M) |
| P vs M | write_p50_us | 1412011 | 32879 | 1399098 | 15473 | 25695 | 113535 | -0.9% | 0.50 | BELOW FLOOR |
| P vs M | write_p99_us | 1536548 | 51008 | 1534156 | 33656 | 43212 | 163904 | -0.2% | 0.06 | BELOW FLOOR |
| P vs M | commits_per_s | 0.71 | 0.02 | 0.71 | 0.01 | 0.01 | 0.07 | +0.8% | 0.45 | BELOW FLOOR |
| P vs M | pss_mib | 488.2 | 1.28 | 487.7 | 1.71 | 1.51 | 28.4 | -0.1% | 0.32 | BELOW FLOOR |
| P vs M | pss_growth_bytes_per_key_read | 54869 | 11228 | 48125 | 13668 | 12508 | 227681 | -12.3% | 0.54 | REFUSED (warm-up MAD above 15% of the median on P and M) |

