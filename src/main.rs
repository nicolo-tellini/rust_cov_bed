use rust_htslib::bam::{Read, IndexedReader, record::Cigar};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(author, version, about, arg_required_else_help = true)]
struct Args {
    #[arg(short, long)]
    bam: String,

    #[arg(short, long)]
    bed: String,

    /// -q
    #[arg(long, default_value_t = 0)]
    min_mapq: u8,

    /// -Q
    #[arg(long, default_value_t = 0)]
    min_baseq: u8,

    /// -f
    #[arg(long, default_value_t = 0)]
    incl_flags: u16,

    /// -F (default samtools-like)
    #[arg(long, default_value_t = 0x704)]
    excl_flags: u16,

    #[arg(long, default_value_t = false)]
    proper_pair: bool,
}

#[derive(Debug)]
struct Region {
    chrom: String,
    start: u64,
    end: u64,
}

fn read_bed(path: &str) -> Result<Vec<Region>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut regions = Vec::new();

    for line in reader.lines() {
        let l = line?;
        if l.starts_with('#') || l.is_empty() { continue; }
        let f: Vec<&str> = l.split_whitespace().collect();
        if f.len() < 3 { continue; }

        regions.push(Region {
            chrom: f[0].to_string(),
            start: f[1].parse()?,
            end: f[2].parse()?,
        });
    }
    Ok(regions)
}

fn pass_filters(rec: &rust_htslib::bam::Record, args: &Args) -> bool {
    if rec.is_unmapped() { return false; }
    if rec.mapq() < args.min_mapq { return false; }

    let flags = rec.flags();

    if (flags & args.excl_flags) != 0 { return false; }
    if args.incl_flags != 0 && (flags & args.incl_flags) == 0 { return false; }
    if args.proper_pair && !rec.is_proper_pair() { return false; }

    true
}

fn compute_region(
    bam_path: &str,
    r: &Region,
    args: &Args,
) -> Result<(u64, u64, f64, f64, f64), Box<dyn Error>> {

    let mut bam = IndexedReader::from_path(bam_path)?;
    let header = bam.header().to_owned();

    let tid = header.tid(r.chrom.as_bytes())
        .ok_or("chrom not found")?;

    bam.fetch((tid, r.start, r.end))?;

    let len = (r.end - r.start + 1) as usize;

    // prefix-sum array
    let mut events = vec![0i32; len + 1];

    let mut num_reads = 0;
    let mut sum_mapq = 0u64;

    let mut sum_baseq = 0u64;
    let mut n_bases = 0u64;

    for rec in bam.records() {
        let rec = rec?;
        if !pass_filters(&rec, args) { continue; }

        let rec_start = rec.pos() as i64;
        let rec_end = rec.cigar().end_pos() as i64;

        if rec_end >= r.start as i64 && rec_start <= r.end as i64 {
            num_reads += 1;
            sum_mapq += rec.mapq() as u64;
        }

        let qual = rec.qual();
        let mut ref_pos = rec.pos() as i64;
        let mut read_pos = 0usize;

        for cig in rec.cigar().iter() {
            match cig {

                Cigar::Match(l) | Cigar::Equal(l) | Cigar::Diff(l) => {
                    let l = *l as i64;

                    let mut seg_start: Option<usize> = None;

                    for i in 0..l {
                        let ref_p = ref_pos + i;

                        if ref_p < r.start as i64 || ref_p > r.end as i64 {
                            if let Some(s) = seg_start {
                                let e = (ref_p - r.start as i64) as usize;
                                events[s] += 1;
                                events[e] -= 1;
                                seg_start = None;
                            }
                            continue;
                        }

                        let q = qual[read_pos + i as usize];

                        if q >= args.min_baseq {
                            let idx = (ref_p - r.start as i64) as usize;

                            if seg_start.is_none() {
                                seg_start = Some(idx);
                            }

                            sum_baseq += q as u64;
                            n_bases += 1;

                        } else {
                            if let Some(s) = seg_start {
                                let e = (ref_p - r.start as i64) as usize;
                                events[s] += 1;
                                events[e] -= 1;
                                seg_start = None;
                            }
                        }
                    }

                    if let Some(s) = seg_start {
                        let e = ((ref_pos + l) - r.start as i64) as usize;
                        events[s] += 1;
                        events[e] -= 1;
                    }

                    ref_pos += l;
                    read_pos += l as usize;
                }

                Cigar::Ins(l) | Cigar::SoftClip(l) => {
                    read_pos += *l as usize;
                }

                Cigar::Del(l) | Cigar::RefSkip(l) => {
                    ref_pos += *l as i64;
                }

                _ => {}
            }
        }
    }

    // prefix sum
    let mut depth = 0i32;
    let mut cov_bases = 0u64;
    let mut sum_depth = 0u64;

    for i in 0..len {
        depth += events[i];
        if depth > 0 { cov_bases += 1; }
        sum_depth += depth as u64;
    }

    let total = len as u64;

    let mean_depth = sum_depth as f64 / total as f64;

    let mean_mapq = if num_reads > 0 {
        sum_mapq as f64 / num_reads as f64
    } else { 0.0 };

    let mean_baseq = if n_bases > 0 {
        sum_baseq as f64 / n_bases as f64
    } else { 0.0 };

    Ok((num_reads, cov_bases, mean_depth, mean_baseq, mean_mapq))
}

fn main() -> Result<(), Box<dyn Error>> {

    let args = Args::parse();
    let regions = read_bed(&args.bed)?;

    println!("#rname\tstartpos\tendpos\tnumreads\tcovbases\tcoverage\tmeandepth\tmeanbaseq\tmeanmapq");

    for r in &regions {
        let (n, c, d, bq, mq) =
            compute_region(&args.bam, r, &args)?;

        let total = r.end - r.start + 1;
        let breadth = (c as f64 / total as f64) * 100.0;

        println!(
            "{}\t{}\t{}\t{}\t{}\t{:.1}\t{:.4}\t{:.1}\t{:.1}",
            r.chrom,
            r.start,
            r.end,
            n,
            c,
            breadth,
            d,
            bq,
            mq
        );
    }

    Ok(())
}
