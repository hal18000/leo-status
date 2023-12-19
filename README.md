# leo-status

leo-status is a tool for monitoring a Leo Bodnar GPSDO

The GPSDOs supported are:
- [Leo Bodnar - Precision GPS Reference Clock](https://www.leobodnar.com/shop/index.php?main_page=product_info&cPath=107&products_id=234)
- [Leo Bodnar - Mini Precision GPS Reference Clock](https://www.leobodnar.com/shop/index.php?main_page=product_info&cPath=107&products_id=301) [UNTESTED]

| Platform     | Tested | Status  |
| ------------ | ------ | ------- |
| MacOS 14.1.2 | Tested | Working |
| Windows 11   | Tested | Working |
| Ubuntu 22.04 | Tested | Working |

## Usage

Please note this program is currently a work in progress. Data is currently printed to the console in a debug format.

Quick start:

```shell
cargo run -- --interval 1s
```

Which will give you:

```
device configuration: GpsdoConfig { output1: true, output2: true, level: 0, fin: 4687500, n3: 11, n2_hs: 11, n2_ls: 1152, n1_hs: 9, nc1_ls: 12, nc2_ls: 24, skew: 0, bw: 15 }, f3 426136, fout1 50000000, fout2 25000000
Using device with serial number AAAABBBCCC
config: GpsdoConfig { output1: true, output2: true, level: 0, fin: 4687500, n3: 11, n2_hs: 11, n2_ls: 1152, n1_hs: 9, nc1_ls: 12, nc2_ls: 24, skew: 0, bw: 15 }, status: GpsdoStatus { loss_count: 0, sat_lock: true, pll_lock: false, locked: false }
config: GpsdoConfig { output1: true, output2: true, level: 0, fin: 4687500, n3: 11, n2_hs: 11, n2_ls: 1152, n1_hs: 9, nc1_ls: 12, nc2_ls: 24, skew: 0, bw: 15 }, status: GpsdoStatus { loss_count: 0, sat_lock: true, pll_lock: false, locked: false }
config: GpsdoConfig { output1: true, output2: true, level: 0, fin: 4687500, n3: 11, n2_hs: 11, n2_ls: 1152, n1_hs: 9, nc1_ls: 12, nc2_ls: 24, skew: 0, bw: 15 }, status: GpsdoStatus { loss_count: 1, sat_lock: false, pll_lock: false, locked: false }
```

For more usage advice, issue the `--help` command.

```
Usage: leo-status [OPTIONS] --interval <INTERVAL>

Options:
      --interval <INTERVAL>            Interval to poll the GPSDO for status
      --serial-number <SERIAL_NUMBER>  Serial number of the Leo Bodnar GPSDO device to use, if not specified any Leo Bodnar GPSDO connected will be used
      --stdout                         Print status of GPSDO to the console in JSON format
  -h, --help                           Print help
  -V, --version                        Print version
```
## Disclaimer

Please note that this program has no guarantees, nor does it have any endorsement or relation to the Leo Bodnar company. You use this program at your own risk, and the author nor Leo Bodnar company are responsible for it's operation (or the lack thereof).

Please ensure you familiarise yourself with the [LICENSE](./LICENSE) for further details.
