#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import unittest
import uuid
import time

from d4message import D4Message, from_bytes


class SimpleTest(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.protocol_version = 1
        cls.packet_type = 1
        cls.sensor_uuid = uuid.uuid4()
        cls.key = b'My Hmac key'
        cls.message = b'blah'

    def test_hmac(self):
        m = D4Message(self.protocol_version, self.packet_type,
                      self.sensor_uuid.bytes, self.key, self.message)
        self.assertTrue(m.validate_hmac(self.key))
        self.assertFalse(m.validate_hmac(b'2'))

    def test_encode_decode(self):
        m = D4Message(self.protocol_version, self.packet_type,
                      self.sensor_uuid.bytes, self.key, self.message)
        now = int(time.time())
        diff = now - m.header.timestamp
        self.assertTrue(diff <= 2, diff)

        encoded = bytes(m.to_bytes())
        decoded = from_bytes(encoded)
        self.assertTrue(decoded.validate_hmac(self.key))
        self.assertEqual(m.to_bytes(), decoded.to_bytes())

    def test_class_members(self):
        m = D4Message(self.protocol_version, self.packet_type,
                      self.sensor_uuid.bytes, self.key, self.message)
        self.assertEqual(m.header.protocol_version, self.protocol_version)
        self.assertEqual(m.header.packet_type, self.packet_type)
        self.assertEqual(bytes(m.header.uuid), self.sensor_uuid.bytes)
        self.assertEqual(m.header.size, len(self.message))
        self.assertEqual(bytes(m.body), self.message)
