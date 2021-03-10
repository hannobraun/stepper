# Stepper - Changelog

## v0.4.1 (2021-01-29)

- Update documentation ([#69], [#71], [#76], [#77])
- Move project to the Flott GitHub organization ([#70])
- Change main branch to `main` ([#74])
- Set up sponsorships ([#75])

[#69]: https://github.com/flott-motion/stepper/pull/69
[#70]: https://github.com/flott-motion/stepper/pull/70
[#71]: https://github.com/flott-motion/stepper/pull/71
[#74]: https://github.com/flott-motion/stepper/pull/74
[#75]: https://github.com/flott-motion/stepper/pull/75
[#76]: https://github.com/flott-motion/stepper/pull/76
[#77]: https://github.com/flott-motion/stepper/pull/77


## v0.4.0 (2021-01-06)

- Fix documentation on docs.rs (hopefully) ([#24])
- Improve `Direction` enum ([#25], [#52])
- Improve API for working with microstepping modes ([#27], [#64])
- Fix and improve documentation ([#28], [#29], [#32], [#65])
- Make driver crates more light-weight ([#35])
- Add new `Driver` struct to serve as abstract API ([#46], [#54], [#56], [#59], [#62])
- Separate setting direction and making steps ([#53])
- Make interface non-blocking ([#61], [#57])

[#24]: https://github.com/flott-motion/stepper/pull/24
[#25]: https://github.com/flott-motion/stepper/pull/25
[#27]: https://github.com/flott-motion/stepper/pull/27
[#28]: https://github.com/flott-motion/stepper/pull/28
[#29]: https://github.com/flott-motion/stepper/pull/29
[#32]: https://github.com/flott-motion/stepper/pull/32
[#35]: https://github.com/flott-motion/stepper/pull/35
[#46]: https://github.com/flott-motion/stepper/pull/46
[#52]: https://github.com/flott-motion/stepper/pull/52
[#53]: https://github.com/flott-motion/stepper/pull/53
[#54]: https://github.com/flott-motion/stepper/pull/54
[#56]: https://github.com/flott-motion/stepper/pull/56
[#57]: https://github.com/flott-motion/stepper/pull/57
[#59]: https://github.com/flott-motion/stepper/pull/59
[#61]: https://github.com/flott-motion/stepper/pull/61
[#62]: https://github.com/flott-motion/stepper/pull/62
[#64]: https://github.com/flott-motion/stepper/pull/64
[#65]: https://github.com/flott-motion/stepper/pull/65


## v0.3.0 (2020-11-12)

- Extend API to support drivers with other microstepping resolutions ([#15])
- Add support for DRV8825 ([#17])
- Require embedded-hal 1.0.0-alpha.4 ([#16], [#21])

[#15]: https://github.com/flott-motion/stepper/pull/15
[#16]: https://github.com/flott-motion/stepper/pull/16
[#17]: https://github.com/flott-motion/stepper/pull/17
[#21]: https://github.com/flott-motion/stepper/pull/21


## v0.2.1 (2020-10-20)

- Fix build system problem that prevented `stspin220` crate from being published.


## v0.2.0 (2020-10-20)

- Add abstract interface over stepper motor driver libraries ([#8])
- Include STSPIN220 driver in Step/Dir ([#9])
- Initial release of Step/Dir library ([step-dir])

[#8]: https://github.com/flott-motion/stepper/pull/8
[#9]: https://github.com/flott-motion/stepper/pull/9
[step-dir]: https://crates.io/crates/step-dir


## v0.1.0 (2020-10-19)

- Initial release of STSPIN220 Driver ([stspin220])

[stspin220]: https://crates.io/crates/stspin220
