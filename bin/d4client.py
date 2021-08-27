#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import uuid
import os
from pathlib import Path
import socket
import sys

from d4message import D4Message


def dir_path(path: str) -> Path:
    if os.path.isdir(path):
        return Path(path)
    else:
        raise argparse.ArgumentTypeError(f"readable_dir:{path} is not a valid path")


def main():
    parser = argparse.ArgumentParser(description='d4 - d4 client.')
    parser.add_argument('-c', '--config-directory', default='conf.sample', type=dir_path)
    args = parser.parse_args()

    with (args.config_directory / 'uuid').open() as _f:
        sensor_uuid = uuid.UUID(_f.read().strip())
    with (args.config_directory / 'key').open() as _f:
        key = _f.read().strip()
    with (args.config_directory / 'version').open() as _f:
        protocol_version = int(_f.read().strip())
    with (args.config_directory / 'type').open() as _f:
        packet_type = int(_f.read().strip())
    with (args.config_directory / 'destination').open() as _f:
        destination = _f.read().strip()

    stdin_message = sys.stdin.read().encode()
    message = D4Message(protocol_version, packet_type, sensor_uuid.bytes,
                        key.encode(), stdin_message)

    if destination == 'stdout':
        sys.stdout.buffer.write(bytes(message.to_bytes()))
    else:
        host, port = destination.split(':')
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.connect((host, int(port)))
            s.sendall(bytes(message.to_bytes()))


if __name__ == '__main__':
    main()
