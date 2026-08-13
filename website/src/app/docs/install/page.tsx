import type { Metadata } from "next";
import { H1, P, DocsShell, Note, H2 } from "@/components/Sections";

export const metadata: Metadata = {
  title: "Installation",
  description:
    "Install ctx via npm, cargo, or prebuilt binaries. Requires Rust 1.85+ only when building from source.",
};

export default function InstallPage() {
  return (
    <DocsShell>
      <H1>Installation</H1>
      <P>
        ctx is a single binary. The quickest way to get it is npm, which avoids
        needing a Rust toolchain. Prebuilt binaries for macOS, Linux, and
        Windows are attached to every GitHub release.
      </P>

      <H2>Via npm (recommended)</H2>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`npm install -g ctxai-cli
ctx --version`}
      </pre>
      <P>
        The npm package ships platform-specific binaries for macOS (x64 +
        arm64), Linux (x64 + arm64), and Windows (x64 + arm64). This is the same
        package the MCP examples use with <span className="font-mono text-sm text-ink">npx -y ctxai-cli mcp</span>.
      </P>

      <H2>Via cargo</H2>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`cargo install ctxai-cli --locked`}
      </pre>
      <P>
        Requires Rust 1.85 or newer. <span className="font-mono text-sm text-ink">--locked</span> pins
        the dependency graph to the versions ctx was tested against.
      </P>

      <H2>Prebuilt binaries</H2>
      <P>
        Download a binary from the{" "}
        <a
          href="https://github.com/halloffame12/CTX/releases"
          target="_blank"
          rel="noreferrer"
          className="font-semibold text-accent-deep hover:underline"
        >
          GitHub releases page
        </a>
        . Each release includes a <span className="font-mono text-sm text-ink">checksums.txt</span> so you
        can verify the download.
      </P>

      <Note>
        All installation methods produce the same <span className="font-mono text-sm text-ink">ctx</span>{" "}
        binary. There is no separate runtime, daemon, or service — the MCP server
        runs as a child process of your AI client.
      </Note>

      <H2>Quick smoke test</H2>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`cd /path/to/your/project
ctx init
ctx doctor`}
      </pre>
      <P>
        <span className="font-mono text-sm text-ink">ctx doctor</span> verifies the
        index and reports anything out of date.
      </P>
    </DocsShell>
  );
}