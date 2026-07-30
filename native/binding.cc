// SKC-LISP-WORLD: Node.js Native Binding for ASM Validators
// Wraps mutation_validator.asm and digest-verifier.asm

#include <node.h>
#include <v8.h>
#include <cstring>
#include <cstdint>
#include <dlfcn.h>

using v8::FunctionCallbackInfo;
using v8::Isolate;
using v8::Local;
using v8::Object;
using v8::String;
using v8::Value;
using v8::Number;
using v8::Boolean;
using v8::ArrayBuffer;
using v8::Uint8Array;

// ============================================================================
// Function pointers to ASM procedures
// ============================================================================

typedef int (*MutationValidateGate)(
    void* mutation_event,
    void* object_store,
    void* validation_result
);

typedef int (*Blake3Verify)(
    void* payload,
    uint64_t payload_length,
    void* expected_digest,
    void* verification_result
);

typedef int (*Ed25519Verify)(
    void* message,
    uint64_t message_length,
    void* signature,
    void* public_key,
    void* verification_result
);

static MutationValidateGate mutation_validate_gate = nullptr;
static Blake3Verify blake3_verify = nullptr;
static Ed25519Verify ed25519_verify = nullptr;

// ============================================================================
// Load ASM library
// ============================================================================

void LoadAsmLibrary(const FunctionCallbackInfo<Value>& args) {
    Isolate* isolate = args.GetIsolate();

    if (args.Length() < 1) {
        isolate->ThrowException(v8::Exception::TypeError(
            String::NewFromUtf8(isolate, "Missing library path").ToLocalChecked()
        ));
        return;
    }

    v8::String::Utf8Value lib_path(isolate, args[0]);
    void* lib_handle = dlopen(*lib_path, RTLD_LAZY);

    if (!lib_handle) {
        isolate->ThrowException(v8::Exception::Error(
            String::NewFromUtf8(isolate, dlerror()).ToLocalChecked()
        ));
        return;
    }

    // Load ASM functions
    mutation_validate_gate = (MutationValidateGate)dlsym(lib_handle, "mutation_validate_gate");
    blake3_verify = (Blake3Verify)dlsym(lib_handle, "blake3_verify");
    ed25519_verify = (Ed25519Verify)dlsym(lib_handle, "ed25519_verify");

    if (!mutation_validate_gate || !blake3_verify || !ed25519_verify) {
        isolate->ThrowException(v8::Exception::Error(
            String::NewFromUtf8(isolate, "Failed to load ASM symbols").ToLocalChecked()
        ));
        dlclose(lib_handle);
        return;
    }

    args.GetReturnValue().Set(Boolean::New(isolate, true));
}

// ============================================================================
// MutationValidateGate wrapper
// ============================================================================

void ValidateMutation(const FunctionCallbackInfo<Value>& args) {
    Isolate* isolate = args.GetIsolate();

    if (!mutation_validate_gate) {
        isolate->ThrowException(v8::Exception::Error(
            String::NewFromUtf8(isolate, "ASM library not loaded").ToLocalChecked()
        ));
        return;
    }

    // args[0] = mutation_event (Uint8Array, 64 bytes)
    // args[1] = object_store pointer (Number)
    // args[2] = validation_result (Uint8Array, 2 bytes, output)

    if (args.Length() < 3) {
        isolate->ThrowException(v8::Exception::TypeError(
            String::NewFromUtf8(isolate, "Invalid argument count").ToLocalChecked()
        ));
        return;
    }

    Local<Uint8Array> mutation_event_buf = args[0].As<Uint8Array>();
    uint64_t object_store = args[1]->NumberValue(isolate->GetCurrentContext()).FromJust();
    Local<Uint8Array> result_buf = args[2].As<Uint8Array>();

    void* mutation_event_ptr = mutation_event_buf->Buffer()->GetContents().data();
    void* result_ptr = result_buf->Buffer()->GetContents().data();

    int ret = mutation_validate_gate(
        mutation_event_ptr,
        (void*)object_store,
        result_ptr
    );

    args.GetReturnValue().Set(Number::New(isolate, ret));
}

// ============================================================================
// Blake3Verify wrapper
// ============================================================================

void VerifyBlake3(const FunctionCallbackInfo<Value>& args) {
    Isolate* isolate = args.GetIsolate();

    if (!blake3_verify) {
        isolate->ThrowException(v8::Exception::Error(
            String::NewFromUtf8(isolate, "ASM library not loaded").ToLocalChecked()
        ));
        return;
    }

    if (args.Length() < 3) {
        isolate->ThrowException(v8::Exception::TypeError(
            String::NewFromUtf8(isolate, "Invalid argument count").ToLocalChecked()
        ));
        return;
    }

    Local<Uint8Array> payload_buf = args[0].As<Uint8Array>();
    uint64_t payload_length = payload_buf->Length();
    Local<Uint8Array> expected_digest = args[1].As<Uint8Array>();
    Local<Uint8Array> result_buf = args[2].As<Uint8Array>();

    void* payload_ptr = payload_buf->Buffer()->GetContents().data();
    void* digest_ptr = expected_digest->Buffer()->GetContents().data();
    void* result_ptr = result_buf->Buffer()->GetContents().data();

    int ret = blake3_verify(payload_ptr, payload_length, digest_ptr, result_ptr);

    args.GetReturnValue().Set(Number::New(isolate, ret));
}

// ============================================================================
// Ed25519Verify wrapper
// ============================================================================

void VerifyEd25519(const FunctionCallbackInfo<Value>& args) {
    Isolate* isolate = args.GetIsolate();

    if (!ed25519_verify) {
        isolate->ThrowException(v8::Exception::Error(
            String::NewFromUtf8(isolate, "ASM library not loaded").ToLocalChecked()
        ));
        return;
    }

    if (args.Length() < 4) {
        isolate->ThrowException(v8::Exception::TypeError(
            String::NewFromUtf8(isolate, "Invalid argument count").ToLocalChecked()
        ));
        return;
    }

    Local<Uint8Array> message_buf = args[0].As<Uint8Array>();
    uint64_t message_length = message_buf->Length();
    Local<Uint8Array> signature = args[1].As<Uint8Array>();
    Local<Uint8Array> public_key = args[2].As<Uint8Array>();
    Local<Uint8Array> result_buf = args[3].As<Uint8Array>();

    void* message_ptr = message_buf->Buffer()->GetContents().data();
    void* sig_ptr = signature->Buffer()->GetContents().data();
    void* key_ptr = public_key->Buffer()->GetContents().data();
    void* result_ptr = result_buf->Buffer()->GetContents().data();

    int ret = ed25519_verify(message_ptr, message_length, sig_ptr, key_ptr, result_ptr);

    args.GetReturnValue().Set(Number::New(isolate, ret));
}

// ============================================================================
// Module initialization
// ============================================================================

void Initialize(Local<Object> exports, Local<Object> module, Local<Object> context) {
    Isolate* isolate = context->GetIsolate();

    exports->Set(context,
        String::NewFromUtf8(isolate, "loadAsmLibrary").ToLocalChecked(),
        v8::FunctionTemplate::New(isolate, LoadAsmLibrary)->GetFunction(context).ToLocalChecked()
    ).FromJust();

    exports->Set(context,
        String::NewFromUtf8(isolate, "validateMutation").ToLocalChecked(),
        v8::FunctionTemplate::New(isolate, ValidateMutation)->GetFunction(context).ToLocalChecked()
    ).FromJust();

    exports->Set(context,
        String::NewFromUtf8(isolate, "verifyBlake3").ToLocalChecked(),
        v8::FunctionTemplate::New(isolate, VerifyBlake3)->GetFunction(context).ToLocalChecked()
    ).FromJust();

    exports->Set(context,
        String::NewFromUtf8(isolate, "verifyEd25519").ToLocalChecked(),
        v8::FunctionTemplate::New(isolate, VerifyEd25519)->GetFunction(context).ToLocalChecked()
    ).FromJust();
}

NODE_MODULE(skclisp_native, Initialize)
