## 1. Refactor Granularity Selection Logic

- [x] 1.1 Replace the three-level threshold (`span_days <= 13` / `<= 60` / else) in `build_timeline()` with the five-level cascade: count days (≤14 → daily), else count distinct ISO weeks (≤14 → weekly), else count distinct months (≤14 → monthly), else count distinct quarters (≤14 → quarterly), else yearly
- [x] 1.2 Add quarterly bucket label formatting (`YYYY-Qn`) where quarter = `(month - 1) / 3 + 1`
- [x] 1.3 Add yearly bucket label formatting (`YYYY`)

## 2. Contiguous Bucket Generation

- [x] 2.1 Add contiguous quarterly bucket enumeration: iterate from (year, quarter) of min_date to (year, quarter) of max_date
- [x] 2.2 Add contiguous yearly bucket enumeration: iterate from year of min_date to year of max_date

## 3. Update Tests

- [x] 3.1 Update existing daily/weekly/monthly boundary tests to match new thresholds (daily ≤14 days, weekly ≤14 distinct weeks, monthly ≤14 distinct months)
- [x] 3.2 Add tests for quarterly granularity: selection trigger, label format, contiguous generation with gaps
- [x] 3.3 Add tests for yearly granularity: selection trigger, label format, contiguous generation with gaps
- [x] 3.4 Add boundary tests for level transitions: 14→15 days, 14→15 weeks, 14→15 months, 14→15 quarters
- [x] 3.5 Add ISO week-year boundary test (e.g., dates spanning Dec/Jan where Dec 29+ belongs to ISO week 1 of next year)
