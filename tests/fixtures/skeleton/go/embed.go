package main

import "io"

type Base struct {
    ID string
}

func (b *Base) GetID() string {
    return b.ID
}

type Composed struct {
    Base
    r io.Reader
}

func (c *Composed) Read(p []byte) (int, error) {
    return c.r.Read(p)
}

type onlyiface interface {
    Read(p []byte) (int, error)
}
