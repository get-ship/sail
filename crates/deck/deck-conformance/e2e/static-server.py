#!/usr/bin/env python3
"""Dumb static file server for the deck-conformance static-deploy e2e.

No daemon, no deck, no ship — just files over HTTP, with an explicit JS MIME so
the browser accepts ES modules (strict-MIME checking rejects modules served as
text/plain). Usage: static-server.py <port> <dir>
"""
import http.server
import os
import socketserver
import sys


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".js": "text/javascript",
        ".mjs": "text/javascript",
        ".css": "text/css",
        ".html": "text/html",
    }

    def log_message(self, *args):  # quiet
        pass


def main() -> None:
    port, root = int(sys.argv[1]), sys.argv[2]
    os.chdir(root)
    with socketserver.TCPServer(("127.0.0.1", port), Handler) as httpd:
        httpd.serve_forever()


if __name__ == "__main__":
    main()
