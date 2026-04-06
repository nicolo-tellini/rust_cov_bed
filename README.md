# rust_cov_bed

## IMPORTANT

I am not familiar with RUST.

This was written with OpenAI. (2026). ChatGPT (GPT-5.3) [Large language model]. https://chat.openai.com

---

## HOW DOES IT DIFFER FROM SAMTOOLS COVERAGE

`rust_cov_bed` use as input a BED file, samtools coverage does not at the moment. 

---

## IS THE OUTPUT COMPLETELY IDENTICAL TO SAMTOOLS SOVERAGE?

Not most of the time but it is comparable.

---

## WHY DID I ASK ChatGPT TO WRITE rust_cov_bed?

because I needed it. 

---

## NOTE

`rust_cov_bed` does not contains all the flags available in samtools coverage but just a subset was relevant for my analyses.  

feel free forking and add additional if needed, please do not pull request because, I am not familiar with RUST and I have no time to test additional changes.

---

# How to install 

```bash
git clone https://github.com/nicolo-tellini/rust_cov_bed.git
cd rust_cov_bed
cargo build --release
```

Binary will be located at:
```bash
target/release/rust_cov_bed
```

# Usage

## Show all flags
```bash 
rust_cov_bed --help
```

## Basic usage
```bash
rust_cov_bed --bam sample.bam --bed regions.bed
```

## With filters 
```bash
rust_cov_bed \
  --bam sample.bam \
  --bed regions.bed \
  --min-mapq 30 \
  --min-baseq 30 
```

## Example 1
<img width="1477" height="235" alt="Screenshot from 2026-04-06 13-34-14" src="https://github.com/user-attachments/assets/c9146c11-66e1-426f-ae74-f411f8a294ee" />

## Example 2
<img width="1117" height="741" alt="Screenshot from 2026-04-06 13-38-36" src="https://github.com/user-attachments/assets/388f4106-b4f7-463b-a570-dda997058052" />


## Disclaimer
This software is provided "as is", without any warranties. The code may contain errors or bugs. The author assume no responsibility for any consequences arising from its use.
