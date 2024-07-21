#!/usr/bin/env python3

import argparse
import contextlib
import glob
import os
import platform
import re
import shutil
import subprocess
import sys
from typing import Optional


def fetch_submodule():
    if shutil.which("git") is not None:
        with working_dir(os.path.dirname(os.path.abspath(__file__))):
            subprocess.run(
                ["git", "submodule", "update", "--init", "--recursive"]
            ).check_returncode()
    else:
        err("git is not installed. Skip fetching submodules.")


def err(msg: str):
    print("\033[91mERR \033[0m: " + msg)


def warn(msg: str):
    print("\033[93mWARN\033[0m: " + msg)


def info(msg: str):
    print("\033[92mINFO\033[0m: " + msg)


def rm_f(path):
    try:
        os.remove(path)
    except FileNotFoundError:
        pass


def glob_norm(path, recursive):
    return [os.path.normpath(p) for p in glob.glob(path, recursive=recursive)]


def rm_glob_f(path, exclude=None, recursive=True):
    if exclude is not None:
        for f in list(
            set(glob_norm(path, recursive=recursive))
            - set(glob_norm(exclude, recursive=recursive))
        ):
            rm_f(f)
    else:
        for f in glob.glob(path, recursive=recursive):
            rm_f(f)


@contextlib.contextmanager
def working_dir(path):
    cwd = os.getcwd()
    os.chdir(path)
    try:
        yield
    finally:
        os.chdir(cwd)


@contextlib.contextmanager
def with_env(**kwargs):
    env = os.environ.copy()
    for key, value in kwargs.items():
        os.environ[key] = value
    try:
        yield
    finally:
        os.environ.clear()
        os.environ.update(env)


class Config:
    _platform: str
    release: bool
    target: Optional[str]
    no_examples: bool

    def __init__(self, args):
        self._platform = platform.system()

        if not self.is_windows() and not self.is_macos() and not self.is_linux():
            err(f'platform "{platform.system()}" is not supported.')
            sys.exit(-1)

        self.release = hasattr(args, "release") and args.release
        self.no_examples = hasattr(args, "no_examples") and args.no_examples

        if hasattr(args, "arch") and args.arch is not None:
            if self.is_linux():
                match args.arch:
                    case "arm32":
                        self.target = "armv7-unknown-linux-gnueabihf"
                    case "aarch64":
                        self.target = "aarch64-unknown-linux-gnu"
                    case _:
                        err(f'arch "{args.arch}" is not supported.')
                        sys.exit(-1)
            elif self.is_windows():
                match args.arch:
                    case "aarch64":
                        self.target = "aarch64-pc-windows-msvc"
                    case _:
                        err(f'arch "{args.arch}" is not supported.')
                        sys.exit(-1)
        else:
            self.target = None

    def cargo_command_base(self, subcommands):
        command = []
        if self.target is None:
            command.append("cargo")
            command.extend(subcommands)
        else:
            if self.is_linux():
                command.append("cross")
                command.extend(subcommands)
            else:
                command.append("cargo")
                command.extend(subcommands)
            command.append("--target")
            command.append(self.target)
        if self.release:
            command.append("--release")
        return command

    def cargo_build_command(self, additional_features: Optional[str] = None):
        command = self.cargo_command_base(["build"])
        features = "remote"
        if additional_features is not None:
            features += " " + additional_features
        command.append("--features")
        command.append(features)
        return command

    def is_windows(self):
        return self._platform == "Windows"

    def is_macos(self):
        return self._platform == "Darwin"

    def is_linux(self):
        return self._platform == "Linux"

    def exe_ext(self):
        return ".exe" if self.is_windows() else ""

    def is_pcap_available(self):
        if not self.is_windows():
            return True
        wpcap_exists = os.path.isfile(
            "C:\\Windows\\System32\\wpcap.dll"
        ) and os.path.isfile("C:\\Windows\\System32\\Npcap\\wpcap.dll")
        packet_exists = os.path.isfile(
            "C:\\Windows\\System32\\Packet.dll"
        ) and os.path.isfile("C:\\Windows\\System32\\Npcap\\Packet.dll")

        return wpcap_exists and packet_exists

    def setup_linker(self):
        if not self.is_linux() or self.target is None:
            return

        os.makedirs(".cargo", exist_ok=True)
        with open(".cargo/config", "w") as f:
            if self.target == "armv7-unknown-linux-gnueabihf":
                f.write("[target.armv7-unknown-linux-gnueabihf]\n")
                f.write('linker = "arm-linux-gnueabihf-gcc"\n')
            if self.target == "aarch64-unknown-linux-gnu":
                f.write("[target.aarch64-unknown-linux-gnu]\n")
                f.write('linker = "aarch64-linux-gnu-gcc"\n')


def rust_build(args):
    config = Config(args)

    with working_dir("."):
        command = config.cargo_build_command(args.features)
        if config.no_examples:
            command.append("--workspace")
            command.append("--exclude")
            command.append("examples")
        subprocess.run(command).check_returncode()


def rust_lint(args):
    config = Config(args)

    with working_dir("."):
        command = config.cargo_build_command(args.features)
        command[1] = "clippy"
        if config.no_examples:
            command.append("--workspace")
            command.append("--exclude")
            command.append("examples")
        command.append("--")
        command.append("-D")
        command.append("warnings")
        subprocess.run(command).check_returncode()


def rust_test(args):
    config = Config(args)

    with working_dir("."):
        with with_env(MIRIFLAGS="-Zmiri-disable-isolation"):
            command = (
                config.cargo_command_base(["+nightly", "miri", "nextest", "run"])
                if args.miri
                else config.cargo_command_base(["nextest", "run"])
            )
            features = "test-utilities remote"
            if args.features is not None:
                features += " " + args.features
            command.append("--features")
            command.append(features)
            command.append("--workspace")
            command.append("--exclude")
            command.append("examples")
            if not config.is_pcap_available():
                command.append("--exclude")
                command.append("autd3-link-soem")

            if args.miri:
                command.append("--exclude")
                command.append("autd3")
                command.append("--exclude")
                command.append("autd3-derive")
                command.append("--exclude")
                command.append("autd3-modulation-audio-file")

            subprocess.run(command).check_returncode()


def rust_run(args):
    examples = [
        "soem",
        "remote_soem",
        "twincat",
        "remote_twincat",
        "simulator",
        "lightweight",
        "lightweight_server",
    ]

    if args.target not in examples:
        err(f'example "{args.target}" is not found.')
        info(f"Available examples: {examples}")
        return -1

    features = None
    match args.target:
        case "soem":
            features = "soem"
        case "remote_soem":
            features = "remote_soem"
        case "twincat":
            features = "twincat"
        case "remote_twincat":
            features = "remote_twincat"
        case "simulator":
            features = "simulator"
        case "lightweight":
            features = "lightweight"
        case "lightweight_server":
            features = "lightweight-server"

    with working_dir("./examples"):
        commands = ["cargo", "run"]
        if args.release:
            commands.append("--release")
        commands.append("--bin")
        commands.append(args.target)
        commands.append("--no-default-features")
        if features is not None:
            commands.append("--features")
            commands.append(features)

        subprocess.run(commands).check_returncode()


def rust_clear(_):
    with working_dir("."):
        subprocess.run(["cargo", "clean"]).check_returncode()


def rust_coverage(args):
    with working_dir("."):
        with with_env(
            RUSTFLAGS="-C instrument-coverage",
            LLVM_PROFILE_FILE="%m-%p.profraw",
        ):
            features = "remote test-utilities test"
            command = [
                "cargo",
                "build",
                "--features",
                features,
                "--workspace",
            ]
            subprocess.run(command).check_returncode()
            command[1] = "test"
            subprocess.run(command).check_returncode()

            command = [
                "grcov",
                ".",
                "-s",
                ".",
                "--binary-path",
                "./target/debug",
                "--llvm",
                "--branch",
                "--ignore-not-existing",
                "-o",
                "./coverage",
                "-t",
                args.format,
                "--excl-line",
                r"^\s*((debug_)?assert(_eq|_ne)?!|#\[derive\()|GRCOV_EXCL_LINE|unreachable!|unimplemented!|tracing::|#\[derivative",
                "--ignore",
                "autd3-derive/**/*",
                "--ignore",
                "autd3-link-*/**/*",
                "--ignore",
                "autd3-protobuf/src/pb/*",
                "--ignore",
                "autd3-protobuf/src/pb/**/*",
                "--ignore",
                "**/soem_bindings/*",
                "--ignore",
                "*/build.rs",
                "--ignore",
                "examples/**/*.rs",
                "--excl-start",
                "GRCOV_EXCL_START",
                "--excl-stop",
                "GRCOV_EXCL_STOP",
            ]
            subprocess.run(command).check_returncode()
            rm_glob_f("**/*.profraw")


def util_update_ver(args):
    version = args.version

    with working_dir("."):
        for toml in glob.glob("./**/*/Cargo.toml", recursive=True):
            with open(toml, "r") as f:
                content = f.read()
                content = re.sub(
                    r'^version = "(.*?)"',
                    f'version = "{version}"',
                    content,
                    flags=re.MULTILINE,
                )
                content = re.sub(
                    r'^autd3(.*)version = "(.*?)"',
                    f'autd3\\1version = "{version}"',
                    content,
                    flags=re.MULTILINE,
                )
            with open(toml, "w") as f:
                f.write(content)


def command_help(args):
    print(parser.parse_args([args.command, "--help"]))


if __name__ == "__main__":
    fetch_submodule()

    with working_dir(os.path.dirname(os.path.abspath(__file__))):
        parser = argparse.ArgumentParser(description="autd3 library build script")
        subparsers = parser.add_subparsers()

        # build
        parser_build = subparsers.add_parser("build", help="see `build -h`")
        parser_build.add_argument(
            "--release", action="store_true", help="release build"
        )
        parser_build.add_argument(
            "--arch", help="cross-compile for specific architecture (for Linux)"
        )
        parser_build.add_argument(
            "--no-examples", action="store_true", help="skip building examples"
        )
        parser_build.add_argument(
            "--features", help="additional features", default=None
        )
        parser_build.set_defaults(handler=rust_build)

        # lint
        parser_lint = subparsers.add_parser("lint", help="see `lint -h`")
        parser_lint.add_argument("--release", action="store_true", help="release build")
        parser_lint.add_argument("--features", help="additional features", default=None)
        parser_lint.add_argument(
            "--no-examples", action="store_true", help="skip examples"
        )
        parser_lint.set_defaults(handler=rust_lint)

        # test
        parser_test = subparsers.add_parser("test", help="see `test -h`")
        parser_test.add_argument("--release", action="store_true", help="release build")
        parser_test.add_argument("--features", help="additional features", default=None)
        parser_test.add_argument("--miri", action="store_true", help="run with miri")
        parser_test.set_defaults(handler=rust_test)

        # run
        parser_run = subparsers.add_parser("run", help="see `run -h`")
        parser_run.add_argument("target", help="binary target")
        parser_run.add_argument("--release", action="store_true", help="release build")
        parser_run.set_defaults(handler=rust_run)

        # clear
        parser_clear = subparsers.add_parser("clear", help="see `clear -h`")
        parser_clear.set_defaults(handler=rust_clear)

        # coverage
        parser_cov = subparsers.add_parser("cov", help="see `cov -h`")
        parser_cov.add_argument(
            "--format", help="output format (lcov|html|text)", default="lcov"
        )
        parser_cov.add_argument("--open", action="store_true", help="open")
        parser_cov.set_defaults(handler=rust_coverage)

        # util
        parser_util = subparsers.add_parser("util", help="see `util -h`")
        subparsers_util = parser_util.add_subparsers()

        # util update version
        parser_util_upver = subparsers_util.add_parser(
            "upver", help="see `util upver -h`"
        )
        parser_util_upver.add_argument("version", help="version")
        parser_util_upver.set_defaults(handler=util_update_ver)

        # help
        parser_help = subparsers.add_parser("help", help="see `help -h`")
        parser_help.add_argument("command", help="command name which help is shown")
        parser_help.set_defaults(handler=command_help)

        args = parser.parse_args()
        if hasattr(args, "handler"):
            args.handler(args)
        else:
            parser.print_help()
