use std::{fs, iter};
use std::collections::HashSet;
use std::time::Duration;
use wcloud::{Tokenizer, WordCloud, WordCloudSize, Word, DEFAULT_EXCLUDE_WORDS_TEXT, sat};
use nanorand::{Rng, WyRand};
use palette::{Srgb, Hsl, IntoColor};
use image::Rgba;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput, BenchmarkId};
use wcloud::sat::to_summed_area_table;


pub fn wcloud(c: &mut Criterion) {
    let mut group = c.benchmark_group("create star wars");
    group.sample_size(10);

    let script_path = "examples/custom_colors/a_new_hope.txt";
    let script_text = fs::read_to_string(script_path)
        .expect("Unable to find a_new_hope.txt")
        .replace("HAN", "Han")
        .replace("LUKE'S", "Luke");

// wordcloud_image.save("examples/custom_colors/a_new_hope.png")
//     .expect("Unable to save image to examples/a_new_hope.png");

    group.bench_function("generate wordcloud", |b| b.iter(|| {
        let mut filter = DEFAULT_EXCLUDE_WORDS_TEXT.lines()
            .collect::<HashSet<_>>();

        filter.insert("int");
        filter.insert("ext");

        let tokenizer = Tokenizer::default()
            .with_max_words(1000)
            .with_filter(filter);

        let wordcloud = WordCloud::default()
            .with_tokenizer(tokenizer)
            .with_word_margin(10)
            .with_rng_seed(1);

        let mask_path = "examples/custom_colors/stormtrooper_mask.png";
        let mask_image = image::open(mask_path).unwrap().to_luma8();
        let mask = WordCloudSize::FromMask(mask_image);

        let color_func = |_word: &Word, rng: &mut WyRand| {
            let lightness = 0.4 + 0.6 * rng.generate::<f32>();

            let col = Hsl::new(0.0, 0.0, lightness);
            let rgb: Srgb = col.into_color();

            let raw: [u8; 3] = rgb.into_format().into();

            Rgba([raw[0], raw[1], raw[2], 255])
        };

        wordcloud.generate_from_text_with_color_func(&script_text, mask, black_box(1.0), color_func)
    }));

    group.finish();
}

pub fn sat(c: &mut Criterion) {
    let mut group = c.benchmark_group("summed area table");
    group.sample_size(10);

    let table_sizes = [(10, 10), (100, 100), (1000, 1000), (1920, 1080), (2560, 1440), (3840, 2160)];
    let mut rng = WyRand::new_seed(1);

    for size in table_sizes {
        group.throughput(Throughput::Bytes(size.0 * size.1));
        group.bench_with_input(BenchmarkId::from_parameter(size.0 * size.1), &(size.0 * size.1), |b, &table_len| {
            let mut table: Vec<u32> = (0..table_len).map(|_| rng.generate_range(0..=255u32))
                .collect();
            b.iter(|| sat::to_summed_area_table(&mut table, size.0 as usize, 0));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    wcloud,
    sat,
);
criterion_main!(benches);
