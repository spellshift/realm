package builderpb

type BuildTaskTomeConfig struct {
	TomeID int64             `json:"tome_id"`
	Params map[string]string `json:"params"`
}
