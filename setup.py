#!/usr/bin/env python3
# -*- coding: utf-8 -*-

from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="d4message",
    version="1.0",
    rust_extensions=[RustExtension("d4message.d4message", binding=Binding.PyO3)],
    packages=["d4message"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
