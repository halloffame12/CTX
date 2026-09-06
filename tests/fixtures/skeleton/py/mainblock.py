import argparse
import os

LOG_LEVEL = os.getenv("LOG_LEVEL", "info")


def run_server(port: int) -> None:
    print(f"listening on {port}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()
    run_server(port=args.port)