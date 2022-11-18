use benchmark_simple::*;
use xoodyak::*;

fn main() {
    let bench = Bench::new();
    let options = &Options {
        iterations: 250_000,
        warmup_iterations: 25_000,
        min_samples: 5,
        max_samples: 10,
        max_rsd: 1.0,
        ..Default::default()
    };

    {
        let mut out = [0u8; 48];
        let mut st = Xoodoo::default();
        let res = bench.run(options, || {
            st.permute();
            st.bytes(&mut out);
            out
        });
        println!("Xoodoo  permutation: {}", res.throughput(out.len() as _));
    }

    {
        let mut out = [0u8; 64];
        let mut st = XoodyakHash::new();
        let res = bench.run(options,  || {
            st.absorb(b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. ");
            st.squeeze(&mut out);
            out
        });
        println!("Xoodyak hash       : {}", res.throughput(out.len() as _));
    }

    {
        let mut out = [0u8; 64];
        let mut st = XoodyakKeyed::new(b"key", None, None, None).unwrap();
        let res = bench.run(options,  || {
            st.absorb(b"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. ");
            st.squeeze(&mut out);
            out
        });
        println!("Xoodyak keyed      : {}", res.throughput(out.len() as _));
    }
}
