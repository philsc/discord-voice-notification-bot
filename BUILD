load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@io_bazel_rules_docker//lang:image.bzl", "app_layer")
load("@io_bazel_rules_docker//container:container.bzl", "container_bundle")

rust_binary(
    name = "bot",
    srcs = ["bot.rs"],
    deps = [
        "@crate_index//:tokio",
        "@crate_index//:serenity",
    ],
)

app_layer(
    name = "bot_image",
    base = "@docker_cc_debian11_base//image",
    binary = ":bot",
)

container_bundle(
    name = "bot_image_bundle",
    images = {
        "philsc.net/discord-voice-notification-bot:latest": ":bot_image",
    },
)

sh_binary(
    name = "deploy",
    srcs = ["deploy.sh"],
    data = [
        ":bot_image_bundle.tar",
    ],
)
