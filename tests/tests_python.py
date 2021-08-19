#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import unittest

from d4message import D4Message, from_bytes


class SimpleTest(unittest.TestCase):

    def test_hmac(self):
        m = D4Message(1, 1, b'1111111111111111', b'1', b'blahh')
        self.assertTrue(m.validate_hmac(b'1'))
        self.assertFalse(m.validate_hmac(b'2'))

    def test_encode_decode(self):
        m = D4Message(1, 1, b'1111111111111111', b'1', b'blahh')
        encoded = bytes(m.to_bytes())
        decoded = from_bytes(encoded)
        self.assertTrue(decoded.validate_hmac(b'1'))
        self.assertEqual(m.to_bytes(), decoded.to_bytes())

    def test_class_members(self):
        m = D4Message(1, 1, b'1111111111111111', b'1', b'blahh')
        self.assertEqual(m.header.protocol_version, 1)
        self.assertEqual(m.header.packet_type, 1)
        self.assertEqual(bytes(m.header.uuid), b'1111111111111111')
        self.assertEqual(m.header.size, 5)
        self.assertEqual(bytes(m.body), b'blahh')
