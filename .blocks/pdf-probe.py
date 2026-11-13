#!/usr/bin/env python3
# /// block
# name = "pdf-probe"
# summary = "Probe GET /api/prescriptions/:id/pdf: PDF magic, A4 MediaBox, filename, QR payload, draft 422."
# effects = "network"
# example = ["--base", "http://127.0.0.1:8080"]
# matches = ['/api/prescriptions', '/api/prescriptions/:id/pdf', '/api/prescriptions/:id/preview']
#
# [returns]
# keys = ["ok", "checks", "base", "prescription_id", "code", "qr_payload"]
# doc = "checks maps probe names to booleans; composes persona-probe --flow rx as a subprocess and reuses the doctor session. pdf checks: signed_pdf 200, application/pdf, %PDF- magic, A4 MediaBox floats within 1 pt of 595.28x841.89, RFC 5987 filename, and the embedded QR bitmap decodes to {base}/verify/{code}. draft_rejected expects 422 on the draft created for the probe."
#
# [params]
# base = { type = "str", required = true, help = "server origin, e.g. http://127.0.0.1:8080" }
# email = { type = "str", default = "probe-rx-doctor@example.com", help = "doctor account created by persona-probe --flow rx" }
# password = { type = "str", default = "ProbePass123!", help = "password shared by the probe accounts" }
# qr-attempts = { type = "int", default = 10, min = 1, max = 20, help = "QR versions tried while decoding the embedded bitmap" }
#
# [provenance]
# created = "2026-11-12"
# source = "spec 08-H"
# ///
"""Probe the prescription PDF route and print one small JSON verdict."""
import argparse
import http.cookiejar
import json
import re
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
import uuid
import zlib

A4_WIDTH = 595.28
A4_HEIGHT = 841.89

M_BLOCKS = {
    1: [16],
    2: [28],
    3: [44],
    4: [32, 32],
    5: [43, 43],
    6: [27, 27, 27, 27],
    7: [31, 31, 31, 31],
    8: [38, 38, 39, 39],
    9: [36, 36, 36, 37, 37],
    10: [43, 43, 43, 43, 44],
}


def run_probe(base, password):
    """Run persona-probe --flow rx and return its verdict dict."""
    cmd = [
        "caveman-blocks", "run", "persona-probe",
        "--base", base, "--password", password, "--flow", "rx",
    ]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
    except (OSError, subprocess.TimeoutExpired):
        return {}
    try:
        return json.loads(proc.stdout.strip().splitlines()[-1])
    except (ValueError, IndexError):
        return {}


def open_url(base, path, jar, accept=None):
    """Open one URL with the cookie jar; return (status, headers, body)."""
    request = urllib.request.Request(base + path)
    if accept:
        request.add_header("Accept", accept)
    opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
    try:
        with opener.open(request, timeout=60) as response:
            return response.status, dict(response.headers), response.read()
    except urllib.error.HTTPError as exc:
        return exc.code, dict(exc.headers), exc.read()
    except OSError:
        return None, {}, b""


def post_json(base, path, jar, payload):
    """POST JSON with a same-origin Origin header; return (status, parsed body)."""
    request = urllib.request.Request(
        base + path,
        data=json.dumps(payload).encode(),
        headers={"Origin": base, "Content-Type": "application/json"},
        method="POST",
    )
    opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
    try:
        with opener.open(request, timeout=30) as response:
            return response.status, json.loads(response.read() or b"{}")
    except urllib.error.HTTPError as exc:
        try:
            return exc.code, json.loads(exc.read() or b"{}")
        except ValueError:
            return exc.code, {}
    except OSError:
        return None, {}


def sign_in(base, email, password):
    """Return a signed-in cookie jar for the doctor account, or None."""
    jar = http.cookiejar.CookieJar()
    status, _ = post_json(
        base, "/api/auth/sign-in/email", jar,
        {"email": email, "password": password, "rememberMe": True},
    )
    return jar if status == 200 else None


def header_value(headers, name):
    """Return a response header value case-insensitively."""
    for key, value in headers.items():
        if key.lower() == name.lower():
            return value
    return ""


def media_box(pdf):
    """Return the (width, height) floats from the first /MediaBox."""
    match = re.search(
        rb"/MediaBox\s*\[\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)\s*\]",
        pdf,
    )
    if not match:
        return None
    values = [float(value) for value in match.groups()]
    return values[2] - values[0], values[3] - values[1]


def disposition_filename(headers):
    """Return the Content-Disposition value, or an empty string."""
    return header_value(headers, "content-disposition")


def image_objects(pdf):
    """Yield (dict_bytes, stream_bytes) for every image XObject in the PDF."""
    for match in re.finditer(rb"(\d+)\s+(\d+)\s+obj\b(.*?)endobj", pdf, re.S):
        body = match.group(3)
        stream_at = body.find(b"stream")
        if stream_at < 0:
            continue
        header = body[:stream_at]
        if b"/Image" not in header:
            continue
        start = stream_at + len(b"stream")
        if body[start:start + 2] == b"\r\n":
            start += 2
        elif body[start:start + 1] in (b"\n", b"\r"):
            start += 1
        end = body.find(b"endstream", start)
        if end < 0:
            continue
        yield header, body[start:end]


def square_grayscale(header, stream):
    """Return (width, height, bits, row_bytes, pixels) for the largest square image."""
    width = re.search(rb"/Width\s+(\d+)", header)
    height = re.search(rb"/Height\s+(\d+)", header)
    bits = re.search(rb"/BitsPerComponent\s+(\d+)", header)
    if not width or not height:
        return None
    width, height = int(width.group(1)), int(height.group(1))
    bits = int(bits.group(1)) if bits else 8
    if width != height or width < 200 or bits not in (1, 8):
        return None
    if b"/FlateDecode" in header:
        try:
            stream = zlib.decompress(stream)
        except zlib.error:
            return None
    channels = 1 if b"/DeviceGray" in header else 3
    row_bytes = (width * bits * channels + 7) // 8
    if len(stream) < row_bytes * height:
        return None
    return width, height, bits, row_bytes, stream


def dark_at(width, bits, row_bytes, stream, channels, x, y):
    """Return True when the pixel at (x, y) is dark."""
    row = y * row_bytes
    if bits == 1:
        value = stream[row + (x // 8)]
        return ((value >> (7 - (x % 8))) & 1) == 0
    offset = row + x * channels
    if channels == 1:
        value = stream[offset]
    else:
        value = (stream[offset] + stream[offset + 1] + stream[offset + 2]) // 3
    return value < 128


def sample_matrix(width, height, bits, row_bytes, channels, stream, version, quiet):
    """Sample the QR module matrix for one version and quiet-zone width."""
    size = 17 + 4 * version
    scale = width / (size + 2 * quiet)
    matrix = []
    for row in range(size):
        y = int((quiet + row + 0.5) * scale)
        y = min(max(y, 0), height - 1)
        line = []
        for col in range(size):
            x = int((quiet + col + 0.5) * scale)
            x = min(max(x, 0), width - 1)
            line.append(dark_at(width, bits, row_bytes, stream, channels, x, y))
        matrix.append(line)
    return matrix


def read_format(matrix, second=False):
    """Read and unmask one 15-bit format word; return (ec, mask) or None."""
    size = len(matrix)

    def get(col, row):
        return 1 if matrix[row][col] else 0

    value = 0
    if not second:
        for index in range(6):
            value = (value << 1) | get(8, index)
        value = (value << 1) | get(8, 7)
        value = (value << 1) | get(8, 8)
        value = (value << 1) | get(7, 8)
        for index in range(9, 15):
            value = (value << 1) | get(14 - index, 8)
    else:
        for index in range(8):
            value = (value << 1) | get(8, size - 1 - index)
        for index in range(7):
            value = (value << 1) | get(size - 7 + index, 8)
    value ^= 0x5412
    ec = (value >> 13) & 0x3
    mask = (value >> 10) & 0x7
    if ec != 0:
        return None
    return mask


def alignment_positions(version):
    """Return the alignment pattern centers for a version."""
    if version == 1:
        return []
    count = version // 7 + 2
    step = 26 if version == 32 else ((version * 4 + count * 2 + 1) // (count * 2 - 2)) * 2
    size = 17 + 4 * version
    positions = [6]
    for index in range(count - 1):
        positions.append(size - 7 - index * step)
    return sorted(positions)


def function_modules(version):
    """Return the reserved-module mask for a QR version."""
    size = 17 + 4 * version
    reserved = [[False] * size for _ in range(size)]

    def mark(row, col):
        if 0 <= row < size and 0 <= col < size:
            reserved[row][col] = True

    for row_start, col_start in ((0, 0), (0, size - 7), (size - 7, 0)):
        for row in range(row_start - 1, row_start + 8):
            for col in range(col_start - 1, col_start + 8):
                mark(row, col)
    for index in range(size):
        mark(6, index)
        mark(index, 6)
    for center_row in alignment_positions(version):
        for center_col in alignment_positions(version):
            if (center_row, center_col) in ((6, 6), (6, size - 7), (size - 7, 6)):
                continue
            for row in range(center_row - 2, center_row + 3):
                for col in range(center_col - 2, center_col + 3):
                    mark(row, col)
    for index in range(9):
        mark(8, index)
        mark(index, 8)
    for index in range(8):
        mark(8, size - 1 - index)
        mark(size - 1 - index, 8)
    mark(size - 8, 8)
    if version >= 7:
        for index in range(6):
            for offset in range(3):
                mark(size - 11 + offset, index)
                mark(index, size - 11 + offset)
    return reserved


def unmask(row, col, mask):
    """Return True when the mask flips the module at (row, col)."""
    if mask == 0:
        return (row + col) % 2 == 0
    if mask == 1:
        return row % 2 == 0
    if mask == 2:
        return col % 3 == 0
    if mask == 3:
        return (row + col) % 3 == 0
    if mask == 4:
        return (row // 2 + col // 3) % 2 == 0
    if mask == 5:
        return (row * col) % 2 + (row * col) % 3 == 0
    if mask == 6:
        return ((row * col) % 2 + (row * col) % 3) % 2 == 0
    return ((row + col) % 2 + (row * col) % 3) % 2 == 0


def codewords(matrix, mask):
    """Read the unmasked data codewords in zigzag order."""
    size = len(matrix)
    reserved = function_modules((size - 17) // 4)
    bits = []
    upward = True
    col = size - 1
    while col > 0:
        if col == 6:
            col -= 1
        rows = range(size - 1, -1, -1) if upward else range(size)
        for row in rows:
            for current in (col, col - 1):
                if reserved[row][current]:
                    continue
                value = matrix[row][current] ^ unmask(row, current, mask)
                bits.append(1 if value else 0)
        col -= 2
        upward = not upward
    output = bytearray()
    for index in range(0, len(bits) - 7, 8):
        byte = 0
        for bit in bits[index:index + 8]:
            byte = (byte << 1) | bit
        output.append(byte)
    return output


def deinterleave(data, version):
    """Return the concatenated data blocks of one EC-M symbol."""
    lengths = M_BLOCKS.get(version)
    if not lengths:
        return b""
    total = sum(lengths)
    stream = data[:total]
    blocks = [[] for _ in lengths]
    position = 0
    for index in range(max(lengths)):
        for block_index, length in enumerate(lengths):
            if index < length and position < len(stream):
                blocks[block_index].append(stream[position])
                position += 1
    return bytes(byte for block in blocks for byte in block)


def parse_payload(data, version):
    """Decode the numeric, alphanumeric, and byte segments of a QR bit stream."""
    alphanumeric = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:"
    bits = "".join(f"{byte:08b}" for byte in data)
    position = 0
    output = ""
    length_bits = 8 if version < 10 else 16
    while position + 4 <= len(bits):
        mode = int(bits[position:position + 4], 2)
        position += 4
        if mode == 0:
            break
        if mode == 1:
            count_bits = 10 if version < 10 else 12
            if position + count_bits > len(bits):
                return ""
            length = int(bits[position:position + count_bits], 2)
            position += count_bits
            while length >= 3:
                if position + 10 > len(bits):
                    return ""
                output += f"{int(bits[position:position + 10], 2):03d}"
                position += 10
                length -= 3
            if length == 2:
                if position + 7 > len(bits):
                    return ""
                output += f"{int(bits[position:position + 7], 2):02d}"
                position += 7
            elif length == 1:
                if position + 4 > len(bits):
                    return ""
                output += f"{int(bits[position:position + 4], 2):01d}"
                position += 4
        elif mode == 2:
            count_bits = 9 if version < 10 else 11
            if position + count_bits > len(bits):
                return ""
            length = int(bits[position:position + count_bits], 2)
            position += count_bits
            for _ in range(length // 2):
                if position + 11 > len(bits):
                    return ""
                value = int(bits[position:position + 11], 2)
                position += 11
                if value // 45 >= len(alphanumeric) or value % 45 >= len(alphanumeric):
                    return ""
                output += alphanumeric[value // 45] + alphanumeric[value % 45]
            if length % 2:
                if position + 6 > len(bits):
                    return ""
                value = int(bits[position:position + 6], 2)
                if value >= len(alphanumeric):
                    return ""
                output += alphanumeric[value]
                position += 6
        elif mode == 4:
            if position + length_bits > len(bits):
                return ""
            length = int(bits[position:position + length_bits], 2)
            position += length_bits
            if position + length * 8 > len(bits):
                return ""
            for _ in range(length):
                output += chr(int(bits[position:position + 8], 2))
                position += 8
        else:
            return ""
    return output


def decode_qr(header, stream, expected, attempts):
    """Decode an image XObject into the expected QR payload."""
    square = square_grayscale(header, stream)
    if not square:
        return ""
    width, height, bits, row_bytes, pixels = square
    channels = 1 if b"/DeviceGray" in header else 3
    for quiet in range(0, 8):
        for version in range(1, min(attempts, 10) + 1):
            matrix = sample_matrix(
                width, height, bits, row_bytes, channels, pixels, version, quiet
            )
            for second in (False, True):
                mask = read_format(matrix, second)
                if mask is None:
                    continue
                data = deinterleave(codewords(matrix, mask), version)
                payload = parse_payload(data, version)
                if payload:
                    if payload == expected:
                        return payload
                    if "/verify/" in payload:
                        return payload
    return ""


def draft_status(base, jar, template_id, patient_id):
    """Create a draft and return the PDF route status for it."""
    status, template = open_json(base, f"/api/templates/{template_id}", jar)
    version_id = template.get("currentVersionId")
    if not version_id or not patient_id or status != 200:
        return None
    status, draft = post_json(
        base, "/api/prescriptions", jar,
        {
            "patientId": patient_id,
            "templateVersionId": version_id,
            "clientId": f"pdf-probe-{uuid.uuid4().hex}",
        },
    )
    if status != 200 or not draft.get("id"):
        return None
    return open_url(base, f"/api/prescriptions/{draft['id']}/pdf", jar)[0]


def open_json(base, path, jar):
    """GET a JSON endpoint; return (status, parsed body)."""
    status, _, body = open_url(base, path, jar, accept="application/json")
    try:
        return status, json.loads(body or b"{}")
    except ValueError:
        return status, {}


def probe(base, email, password, attempts):
    """Run the PDF probe; return the verdict dict."""
    checks = {}
    verdict = run_probe(base, password)
    checks["fixture"] = bool(verdict.get("ok"))
    prescription_id = verdict.get("prescription_id")
    code = verdict.get("code")
    if not checks["fixture"] or not prescription_id or not code:
        return {
            "ok": False,
            "checks": checks,
            "base": base,
            "error": "persona-probe --flow rx failed: " + str(verdict.get("error", "unknown")),
        }
    jar = sign_in(base, email, password)
    checks["doctor_session"] = jar is not None
    if jar is None:
        return {"ok": False, "checks": checks, "base": base, "error": "doctor sign-in failed"}

    status, headers, pdf = open_url(base, f"/api/prescriptions/{prescription_id}/pdf", jar)
    checks["signed_pdf"] = (
        status == 200
        and header_value(headers, "Content-Type").startswith("application/pdf")
        and pdf.startswith(b"%PDF-")
    )
    box = media_box(pdf)
    checks["a4_media_box"] = bool(
        box
        and abs(box[0] - A4_WIDTH) <= 1
        and abs(box[1] - A4_HEIGHT) <= 1
    )
    disposition = disposition_filename(headers)
    checks["rfc5987_filename"] = bool(
        re.search(r"filename\*=UTF-8''", disposition)
        and re.search(r"filename=\"[^\"]+\"", disposition)
        and re.search(r"-receta-\d{2}-\d{2}-\d{4}-[^/\"]+\.pdf", disposition)
    )
    expected = f"{base}/verify/{code}"
    payload = ""
    for header, stream in image_objects(pdf):
        payload = decode_qr(header, stream, expected, attempts)
        if payload:
            break
    checks["qr_payload"] = bool(payload) and (
        urllib.parse.urlsplit(payload).path == f"/verify/{code}"
    )

    draft = draft_status(
        base, jar, verdict.get("template_id"), verdict.get("patient_id")
    )
    checks["draft_rejected"] = draft == 422

    ok = all(checks.values())
    result = {
        "ok": ok,
        "checks": {key: bool(value) for key, value in checks.items()},
        "base": base,
        "prescription_id": prescription_id,
        "code": code,
        "qr_payload": payload,
    }
    if not ok:
        result["error"] = "one or more PDF checks failed"
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    parser.add_argument("--email", default="probe-rx-doctor@example.com")
    parser.add_argument("--password", default="ProbePass123!")
    parser.add_argument("--qr-attempts", type=int, default=10)
    args = parser.parse_args()
    return probe(args.base.rstrip("/"), args.email, args.password, args.qr_attempts)


if __name__ == "__main__":
    try:
        result = main()
    except Exception as exc:
        print(json.dumps({"error": str(exc)}))
        sys.exit(1)
    print(json.dumps(result))
    sys.exit(0 if result.get("ok") else 1)
