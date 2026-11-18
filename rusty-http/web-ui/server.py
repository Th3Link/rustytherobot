#!/usr/bin/env python3
"""Serve static UI and forward /api/* to the unchanged Rust HTTP API."""
import argparse
import http.client
import json
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

ROUTES = {"event": "GET", "robot_list": "GET", "robot_state": "POST", "create": "POST", "remove": "DELETE",
          "rename": "PUT", "move": "PUT", "attack": "PUT", "repair": "PUT", "transfer_charge": "PUT",
          "board_dimensions": "GET", "chargepads": "GET"}


class Handler(SimpleHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self):
        if self.path.startswith('/api/') or self.path.lstrip('/') in ROUTES:
            self.proxy()
        else:
            super().do_GET()

    def do_POST(self):
        self.proxy()

    def do_PUT(self):
        self.proxy()

    def do_DELETE(self):
        self.proxy()

    def proxy(self):
        route = self.path.removeprefix('/api/') if self.path.startswith('/api/') else self.path.lstrip('/')
        if ROUTES.get(route) != self.command:
            self.send_error(405, 'Unknown API route or method')
            return
        connection = http.client.HTTPConnection(self.server.api.hostname, self.server.api.port or 3000, timeout=8)
        try:
            length = int(self.headers.get('Content-Length', 0))
            if length < 0 or length > 65536:
                self.send_error(413)
                return
            body = self.rfile.read(length) if length else None
            connection.request(self.command, '/' + route, body=body, headers={'Content-Type': 'application/json'})
            response = connection.getresponse()
            self.send_response(response.status)
            self.send_header('Content-Type', response.getheader('Content-Type', 'application/json'))
            self.send_header('Cache-Control', 'no-store')
            self.send_header('Connection', 'close')
            self.close_connection = True
            self.end_headers()
            if route == 'event' and response.status == 200:
                if connection.sock is not None:
                    connection.sock.settimeout(None)
                while chunk := response.read1(65536):
                    self.wfile.write(chunk)
                    self.wfile.flush()
            else:
                self.wfile.write(response.read())
        except (BrokenPipeError, ConnectionResetError):
            pass
        except (OSError, ValueError, http.client.HTTPException) as error:
            if not self.close_connection:
                payload = json.dumps({'error': str(error)}).encode()
                self.send_response(502)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
        finally:
            connection.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--port', type=int, default=8080)
    parser.add_argument('--api', default='http://127.0.0.1:3000')
    args = parser.parse_args()
    api = urlsplit(args.api)
    if api.scheme != 'http' or not api.hostname:
        parser.error('--api must be an HTTP origin, e.g. http://127.0.0.1:3000')
    server = ThreadingHTTPServer(('127.0.0.1', args.port), partial(Handler, directory=str(Path(__file__).parent)))
    server.api = api
    print(f'Rusty Arena: http://127.0.0.1:{args.port} → {args.api}', flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.server_close()
