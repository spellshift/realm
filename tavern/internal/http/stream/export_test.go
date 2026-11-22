package stream

// WithOnRegister sets a test hook invoked after a stream is registered in the Mux event loop.
func WithOnRegister(fn func(*Stream)) MuxOption {
	return func(m *Mux) {
		m.onRegister = fn
	}
}
