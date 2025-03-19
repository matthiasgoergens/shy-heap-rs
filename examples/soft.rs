use softheap::pairing::{Heap, Pairing, Pool};
use softheap::schubert::Operation;

use rand::seq::SliceRandom;

pub fn main() {
    let mut rng = rand::rng();
    let mut bytes = "Hello, random!".to_string().into_bytes();
    bytes.shuffle(&mut rng);
    let str = String::from_utf8(bytes).unwrap();
    println!("{}", str);
    /*
    Heap {
        root: Some(Pairing {
            key: Pool { item: 31, count: 9 },
            children: [
                Pairing {
                    key: Pool { item: 33, count: 0 },
                    children: [Pairing {
                        key: Pool { item: 67, count: 0 },
                        children: [],
                    }],
                },
                Pairing {
                    key: Pool { item: 83, count: 1 },
                    children: [Pairing {
                        key: Pool {
                            item: 121,
                            count: 0,
                        },
                        children: [Pairing {
                            key: Pool {
                                item: 147,
                                count: 0,
                            },
                            children: [],
                        }],
                    }],
                },
                Pairing {
                    key: Pool { item: 40, count: 0 },
                    children: [Pairing {
                        key: Pool { item: 90, count: 0 },
                        children: [],
                    }],
                },
                Pairing {
                    key: Pool { item: 56, count: 1 },
                    children: [
                        Pairing {
                            key: Pool {
                                item: 113,
                                count: 0,
                            },
                            children: [],
                        },
                        Pairing {
                            key: Pool {
                                item: 144,
                                count: 0,
                            },
                            children: [],
                        },
                        Pairing {
                            key: Pool { item: 65, count: 0 },
                            children: [],
                        },
                    ],
                },
                Pairing {
                    key: Pool { item: 92, count: 1 },
                    children: [
                        Pairing {
                            key: Pool {
                                item: 108,
                                count: 0,
                            },
                            children: [],
                        },
                        Pairing {
                            key: Pool {
                                item: 115,
                                count: 0,
                            },
                            children: [],
                        },
                        Pairing {
                            key: Pool {
                                item: 119,
                                count: 1,
                            },
                            children: [Pairing {
                                key: Pool {
                                    item: 120,
                                    count: 0,
                                },
                                children: [Pairing {
                                    key: Pool {
                                        item: 143,
                                        count: 0,
                                    },
                                    children: [],
                                }],
                            }],
                        },
                    ],
                },
                Pairing {
                    key: Pool { item: 38, count: 3 },
                    children: [
                        Pairing {
                            key: Pool { item: 50, count: 0 },
                            children: [],
                        },
                        Pairing {
                            key: Pool { item: 47, count: 0 },
                            children: [],
                        },
                        Pairing {
                            key: Pool { item: 68, count: 0 },
                            children: [],
                        },
                        Pairing {
                            key: Pool {
                                item: 127,
                                count: 0,
                            },
                            children: [],
                        },
                        Pairing {
                            key: Pool {
                                item: 111,
                                count: 1,
                            },
                            children: [Pairing {
                                key: Pool {
                                    item: 122,
                                    count: 0,
                                },
                                children: [Pairing {
                                    key: Pool {
                                        item: 160,
                                        count: 0,
                                    },
                                    children: [],
                                }],
                            }],
                        },
                        Pairing {
                            key: Pool { item: 89, count: 4 },
                            children: [
                                Pairing {
                                    key: Pool { item: 99, count: 0 },
                                    children: [Pairing {
                                        key: Pool {
                                            item: 118,
                                            count: 0,
                                        },
                                        children: [],
                                    }],
                                },
                                Pairing {
                                    key: Pool {
                                        item: 105,
                                        count: 0,
                                    },
                                    children: [],
                                },
                                Pairing {
                                    key: Pool {
                                        item: 106,
                                        count: 0,
                                    },
                                    children: [Pairing {
                                        key: Pool {
                                            item: 151,
                                            count: 0,
                                        },
                                        children: [],
                                    }],
                                },
                            ],
                        },
                    ],
                },
                Pairing {
                    key: Pool { item: 51, count: 1 },
                    children: [Pairing {
                        key: Pool {
                            item: 126,
                            count: 0,
                        },
                        children: [
                            Pairing {
                                key: Pool {
                                    item: 159,
                                    count: 0,
                                },
                                children: [],
                            },
                            Pairing {
                                key: Pool {
                                    item: 142,
                                    count: 0,
                                },
                                children: [],
                            },
                        ],
                    }],
                },
                Pairing {
                    key: Pool { item: 96, count: 1 },
                    children: [
                        Pairing {
                            key: Pool {
                                item: 109,
                                count: 0,
                            },
                            children: [],
                        },
                        Pairing {
                            key: Pool {
                                item: 102,
                                count: 0,
                            },
                            children: [Pairing {
                                key: Pool {
                                    item: 135,
                                    count: 0,
                                },
                                children: [],
                            }],
                        },
                    ],
                },
                Pairing {
                    key: Pool { item: 52, count: 1 },
                    children: [Pairing {
                        key: Pool { item: 70, count: 0 },
                        children: [Pairing {
                            key: Pool { item: 88, count: 0 },
                            children: [],
                        }],
                    }],
                },
                Pairing {
                    key: Pool {
                        item: 114,
                        count: 1,
                    },
                    children: [
                        Pairing {
                            key: Pool {
                                item: 137,
                                count: 0,
                            },
                            children: [],
                        },
                        Pairing {
                            key: Pool {
                                item: 136,
                                count: 0,
                            },
                            children: [],
                        },
                        Pairing {
                            key: Pool {
                                item: 129,
                                count: 1,
                            },
                            children: [
                                Pairing {
                                    key: Pool {
                                        item: 154,
                                        count: 0,
                                    },
                                    children: [],
                                },
                                Pairing {
                                    key: Pool {
                                        item: 150,
                                        count: 0,
                                    },
                                    children: [],
                                },
                            ],
                        },
                    ],
                },
                Pairing {
                    key: Pool { item: 71, count: 0 },
                    children: [Pairing {
                        key: Pool { item: 80, count: 0 },
                        children: [Pairing {
                            key: Pool {
                                item: 101,
                                count: 0,
                            },
                            children: [],
                        }],
                    }],
                },
            ],
        }),
    }
    */
}

// Test failed: 81 >= 3 * 26; uncorrupted: 52

// minimal failing input: ops = 10 25 _ 143 120 119 100 92 108 77 115 65 36 19 4 144 113 56 83 22 121 147 33 67 31 29 6 40 13 90 66 8 26 105 18 106 151 58 118 1 99 32 89 122 160 111 42 23 47 38 50 68 _ 127 14 51 11 142 126 159 102 135 9 28 109 96 70 88 52 30 114 137 136 48 154 129 78 5 150 3 71 101 80 _
