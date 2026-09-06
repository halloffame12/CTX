package main

import (
	"fmt"

	"github.com/example/ctxshop/models"
)

func main() {
	fmt.Println(models.Greet())
}

func helper() string {
	return "helper"
}