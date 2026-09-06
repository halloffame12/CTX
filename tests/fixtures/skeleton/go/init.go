package main

var registry = map[string]string{}

func init() {
    registry["a"] = "alpha"
    registry["b"] = "beta"
}

func main() {
    run(registry)
}

type Builder[T any] struct {
    prefix string
    items  []T
}

func (b *Builder[T]) Add(item T) *Builder[T] {
    b.items = append(b.items, item)
    return b
}

func (b Builder[T]) Build() []T {
    return b.items
}

func Variadic(args ...int) int {
    total := 0
    for _, a := range args {
        total += a
    }
    return total
}
