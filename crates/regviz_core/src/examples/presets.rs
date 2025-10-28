#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sample {
    pub input: &'static str,
    pub expected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    pub name: &'static str,
    pub regex: &'static str,
    pub samples: &'static [Sample],
}

pub fn presets() -> &'static [Example] {
    const EXAMPLES: &[Example] = &[
        Example {
            name: "Balanced As",
            regex: "a(b|c)*a",
            samples: &[
                Sample {
                    input: "aba",
                    expected: true,
                },
                Sample {
                    input: "accca",
                    expected: true,
                },
                Sample {
                    input: "aa",
                    expected: true,
                },
                Sample {
                    input: "ab",
                    expected: false,
                },
            ],
        },
        Example {
            name: "AB Repeats or C",
            regex: "(ab)*|c",
            samples: &[
                Sample {
                    input: "",
                    expected: true,
                },
                Sample {
                    input: "ab",
                    expected: true,
                },
                Sample {
                    input: "abab",
                    expected: true,
                },
                Sample {
                    input: "c",
                    expected: true,
                },
                Sample {
                    input: "ac",
                    expected: false,
                },
            ],
        },
        Example {
            name: "A Plus B Optional",
            regex: "a+b?",
            samples: &[
                Sample {
                    input: "a",
                    expected: false,
                },
                Sample {
                    input: "ab",
                    expected: true,
                },
                Sample {
                    input: "aaab",
                    expected: true,
                },
                Sample {
                    input: "aaa",
                    expected: true,
                },
            ],
        },
        Example {
            name: "Ends With ABB",
            regex: "(a+b)*abb",
            samples: &[
                Sample {
                    input: "abb",
                    expected: true,
                },
                Sample {
                    input: "aabb",
                    expected: true,
                },
                Sample {
                    input: "ababa",
                    expected: false,
                },
            ],
        },
        Example {
            name: "Nested Choice",
            regex: "a(bc|d)+e?",
            samples: &[
                Sample {
                    input: "abcde",
                    expected: true,
                },
                Sample {
                    input: "abcbc",
                    expected: false,
                },
            ],
        },
        Example {
            name: "Literal Pipe",
            regex: "a\\|b",
            samples: &[
                Sample {
                    input: "a+b",
                    expected: true,
                },
                Sample {
                    input: "ab",
                    expected: false,
                },
            ],
        },
    ];
    EXAMPLES
}
