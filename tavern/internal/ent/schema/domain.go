package schema

import "entgo.io/ent"

// Domain holds the schema definition for the Domain entity.
type Domain struct {
	ent.Schema
}

// Fields of the Domain.
func (Domain) Fields() []ent.Field {
	return nil
}

// Edges of the Domain.
func (Domain) Edges() []ent.Edge {
	return nil
}
