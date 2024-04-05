# Bitseed

[English](docs/pages/index.en-US.mdx) | [中文](docs/pages/index.zh-CN.mdx)

## Install

```bash
cargo install --path .
```

## Run

Prepare the development environment by following the instructions in the [Dev Guide](./dev_guide.md).

```bash
bitseed generator --fee-rate 1 --name random --generator generator/generator.wasm
bitseed deploy --fee-rate 1 --generator $the_inscription_from_pre_step --tick bits --amount 210000000000 --deploy-args 1000 --deploy-args 100000
bitseed mint --fee-rate 1 --deploy-inscription-id $the_inscription_from_pre_step 
bitseed split --fee-rate 1 --sft-inscription-id $the_inscription_from_pre_step --amounts 500 --amounts 300
bitseed merge --fee-rate 1 --sft-inscription-ids $the_inscription_from_pre_step_0 --sft-inscription-ids $the_inscription_from_pre_step_1 --sft-inscription-ids $the_inscription_from_pre_step_2
bitseed view --sft-inscription-id $the_inscription_from_pre_step
```
## Test

Install docker first.

```bash
make test
```