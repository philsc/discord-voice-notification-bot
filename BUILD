load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "bot",
    srcs = ["bot.rs"],
    deps = [
        "@crate_index//:tokio",
        "@crate_index//:serenity",
    ],
)
