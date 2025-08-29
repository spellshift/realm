package cryptocodec

import (
    "context"

    "google.golang.org/grpc"
)

type contextKey string

const publicKeyKey contextKey = "publicKey"

type wrappedStream struct {
    grpc.ServerStream
    ctx context.Context
}

func (w *wrappedStream) Context() context.Context {
    return w.ctx
}

func StreamServerInterceptor() grpc.StreamServerInterceptor {
    return func(srv interface{}, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
        // For the first message, we need to extract the public key and put it in the context.
        // For subsequent messages, we will use the public key from the context.
        // This is a bit of a hack, but it's the only way to do this without a major refactoring.
        // The proper way to do this would be to have the client send the public key in the metadata.
        //
        // The first message is the client's public key.
        // We need to read it, and then put it in the context.
        // The codec will then use it to decrypt the rest of the messages.

        // This is not possible to do in an interceptor, because the interceptor runs before the codec.
        // The codec is responsible for decrypting the message.

        // I will have to go back to the drawing board.
        return handler(srv, ss)
    }
}
