struct OutputStruct {
    unsigned int status;
    size_t size;
};

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
#  ifdef MODULE_API_EXPORTS
#    define MODULE_API __declspec(dllexport)
#  else
#    define MODULE_API __declspec(dllimport)
#  endif
#else
#  define MODULE_API
#endif

MODULE_API OutputStruct c_convert(
    char *,     // i_buffer_ptr
    size_t,     // i_buffer_len
    size_t,     // max_size
    int,        // transfer_syntax_pre
    int,        // transfer_syntax_post
    int,        // photometric_interpretation
    char,       // is_lossy
    int,        // quality1
    int,        // quality2
    int,        // quality3
    char,       // irreversible
    int         // allow_error
);

#ifdef __cplusplus
}
#endif