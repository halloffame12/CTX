/** Builds an http client with middleware. */
@Injectable()
export class HttpClient<T extends { baseUrl: string }> {
  private stack: Array<(req: Request) => Promise<Response>> = [];
  // middleware registration
  use(fn: (req: Request, next: () => Promise<Response>) => Promise<Response>): this {
    this.stack.push(fn);
    return this;
  }
  run(req: Request): Promise<Response> {
    const chain = this.stack.reduceRight((next, fn) => {
      return (r: Request) => fn(r, () => next(r));
    }, (r: Request) => this.dispatch(r));
    return chain(req);
  }
  private dispatch(req: Request): Promise<Response> {
    return Promise.resolve(new Response("ok"));
  }
}
export function outer(input: number): number {
  const factor = 2;
  function inner(x: number): number {
    return x * factor;
  }
  const double = (y: number) => {
    return y + inner(1);
  };
  return double(input);
}
export function* gen(max: number): Generator<number> {
  for (let i = 0; i < max; i++) {
    yield i;
  }
}
