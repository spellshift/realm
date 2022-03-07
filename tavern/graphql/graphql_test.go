package graphql_test

import "strconv"

// convertID turns a GraphQL string ID response into an int.
func convertID(id string) int {
	if id == "" {
		return 0
	}
	intID, err := strconv.Atoi(id)
	if err != nil {
		panic(err)
	}
	return intID
}
