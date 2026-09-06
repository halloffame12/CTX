package main

import "fmt"

func outer(n int) int {
	factor := 2
	inner := func(x int) int {
		return x * factor
	}
	return inner(n)
}

type Handler struct {
	Next func(req string) string
}

func (h *Handler) Chain(req string) string {
	if h.Next == nil {
		return fmt.Sprintf("root:%s", req)
	}
	return h.Next(req)
}
