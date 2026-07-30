// SKC-LISP-WORLD: Node.js Native Binding for ASM Validators
// Wraps mutation_validator.asm and digest-verifier.asm

#include <node.h>
#include <v8.h>
#include <cstring>
#include <cstdint>

#ifdef _WIN32
  #include <windows.h>
  #define dlopen(x, y) LoadLibraryA(x)
  #define dlsym(x, y) GetProcAddress(reinterpret_cast<HMODULE>(x), y)
  #define dlerror() "LoadLibrary/GetProcAddress failed"
#else
  #include <dlfcn.h>
#endif

using v8::FunctionCallbackInfo;
using v8::Isolate;
using v8::Local;
using v8::Object;
using v8::String;
using v8::Value;
using v8::Number;
using v8::Boolean;
using v8::Context;

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
    Local<Context> ctx = isolate->GetCurrentContext();

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
        return;
    }

    args.GetReturnValue().Set(Boolean::New(isolate, true));
}

// ============================================================================
// MutationValidateGate wrapper
// ============================================================================

void ValidateMutation(const FunctionCallbackInfo<Value>& args) {
    Isolate* isolate = args.GetIsolate();
    Local<Context> ctx = isolate->GetCurrentContext();

    if (!mutation_validate_gate) {
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

    // For now: stub implementation (returns 1 = pass)
    args.GetReturnValue().Set(Number::New(isolate, 1));
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

    // Stub: returns 1 = valid
    args.GetReturnValue().Set(Number::New(isolate, 1));
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

    // Stub: returns 1 = valid
    args.GetReturnValue().Set(Number::New(isolate, 1));
}

// ============================================================================
// Module initialization
// ============================================================================

void Initialize(Local<Object> exports, Local<Object> module, Local<Context> ctx) {
    Isolate* isolate = ctx->GetIsolate();

    NODE_SET_METHOD(exports, "loadAsmLibrary", LoadAsmLibrary);
    NODE_SET_METHOD(exports, "validateMutation", ValidateMutation);
    NODE_SET_METHOD(exports, "verifyBlake3", VerifyBlake3);
    NODE_SET_METHOD(exports, "verifyEd25519", VerifyEd25519);
}

NODE_MODULE(skclisp_native, Initialize)
