package redirectors

import (
	"fmt"

	"google.golang.org/grpc/encoding"
)

func init() {
	encoding.RegisterCodec(gRPCRawCodec{})
}

// gRPCRawCodec passes through raw bytes without marshaling/unmarshaling
type gRPCRawCodec struct{}

func (gRPCRawCodec) Marshal(v any) ([]byte, error) {
	if b, ok := v.([]byte); ok {
		return b, nil
	}
	return nil, fmt.Errorf("failed to marshal, message is %T", v)
}

func (gRPCRawCodec) Unmarshal(data []byte, v any) error {
	if b, ok := v.(*[]byte); ok {
		*b = data
		return nil
	}
	return fmt.Errorf("failed to unmarshal, message is %T", v)
}

func (gRPCRawCodec) Name() string {
	return "raw"
}
