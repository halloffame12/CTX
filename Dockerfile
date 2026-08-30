FROM node:20-alpine

# CTX is distributed as the ctxai-cli npm package and launched over stdio
# (see server.json). This image mirrors that launch command so Glama (and any
# containerized MCP client) can build and introspect the server reproducibly.
ENTRYPOINT ["npx"]
CMD ["-y", "ctxai-cli", "mcp"]
