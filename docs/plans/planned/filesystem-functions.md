# Extended File System Functions

## Status: Planned

## Overview

Implement comprehensive file system operations including file streams, directory operations, file information functions, and stream wrappers.

## Current Status

VHP implements 10 basic file I/O functions. Many advanced file system features are missing.

## Background

File system operations are essential for PHP applications used in:
- Configuration management
- Data import/export
- File uploads
- Logging
- Directory traversal
- File permissions and ownership

## Already Implemented (10 functions)

`file_get_contents`, `file_put_contents`, `file_exists`, `is_file`, `is_dir`, `filemtime`, `filesize`, `unlink`, `is_readable`, `is_writable`

## Functions to Implement

### File Operations

1. **fopen**
   ```php
   fopen($filename, $mode, $use_include_path = false, $context = null): resource|false
   ```
   - Open file or URL
   - Support modes: 'r', 'r+', 'w', 'w+', 'a', 'a+', 'x', 'x+', 'c', 'c+'
   - Support binary mode 'b'

2. **fclose**
   ```php
   fclose($stream): bool
   ```
   - Close open file pointer

3. **fread**
   ```php
   fread($stream, $length): string|false
   ```
   - Binary-safe file read

4. **fwrite**
   ```php
   fwrite($stream, $data, $length = null): int|false
   ```
   - Binary-safe file write
   - Return number of bytes written

5. **fgets**
   ```php
   fgets($stream, $length = null): string|false
   ```
   - Get line from file pointer

6. **fgetc**
   ```php
   fgetc($stream): string|false
   ```
   - Get character from file pointer

7. **fgetcsv**
   ```php
   fgetcsv($stream, $length = 0, $separator = ',', $enclosure = '"', $escape = '\\'): array|false|null
   ```
   - Get line from file pointer and parse for CSV fields

8. **fputcsv**
   ```php
   fputcsv($stream, $fields, $separator = ',', $enclosure = '"', $escape = '\\', $eol = "\n"): int|false
   ```
   - Format line as CSV and write to file pointer

9. **fpassthru**
   ```php
   fpassthru($stream): int
   ```
   - Output all remaining data on a file pointer

10. **feof**
    ```php
    feof($stream): bool
    ```
    - Test for end-of-file on file pointer

11. **fseek**
    ```php
    fseek($stream, $offset, $whence = SEEK_SET): int
    ```
    - Seek on a file pointer
    - Support SEEK_SET, SEEK_CUR, SEEK_END

12. **ftell**
    ```php
    ftell($stream): int|false
    ```
    - Return current position of file pointer

13. **rewind**
    ```php
    rewind($stream): bool
    ```
    - Rewind position of file pointer

14. **fflush**
    ```php
    fflush($stream): bool
    ```
    - Flush output to file

15. **flock**
    ```php
    flock($stream, $operation, &$would_block = null): bool
    ```
    - Portable advisory file locking
    - Support LOCK_SH, LOCK_EX, LOCK_UN, LOCK_NB

16. **ftruncate**
    ```php
    ftruncate($stream, $size): bool
    ```
    - Truncates a file to a given length

### File Information

17. **file**
    ```php
    file($filename, $flags = 0, $context = null): array|false
    ```
    - Reads entire file into array

18. **fileatime**
    ```php
    fileatime($filename): int|false
    ```
    - Gets last access time of file

19. **filectime**
    ```php
    filectime($filename): int|false
    ```
    - Gets inode change time of file

20. **filegroup**
    ```php
    filegroup($filename): int|false
    ```
    - Gets file group

21. **fileinode**
    ```php
    fileinode($filename): int|false
    ```
    - Gets file inode

22. **fileowner**
    ```php
    fileowner($filename): int|false
    ```
    - Gets file owner

23. **fileperms**
    ```php
    fileperms($filename): int|false
    ```
    - Gets file permissions

24. **filesize**
    ```php
    filesize($filename): int|false
    ```
    - Gets file size (already implemented)

25. **filetype**
    ```php
    filetype($filename): string|false
    ```
    - Gets file type

26. **stat**
    ```php
    stat($filename): array|false
    ```
    - Gives information about a file

27. **lstat**
    ```php
    lstat($filename): array|false
    ```
    - Gives information about a file or symbolic link

28. **fstat**
    ```php
    fstat($stream): array|false
    ```
    - Gets information about a file using an open file pointer

29. **touch**
    ```php
    touch($filename, $time = null, $atime = null): bool
    ```
    - Sets access and modification time

30. **clearstatcache**
    ```php
    clearstatcache($clear_realpath_cache = false, $filename = null): void
    ```
    - Clear file status cache

### Directory Operations

31. **opendir**
    ```php
    opendir($path, $context = null): resource|false
    ```
    - Open directory handle

32. **closedir**
    ```php
    closedir($stream): void
    ```
    - Close directory handle

33. **readdir**
    ```php
    readdir($stream): string|false
    ```
    - Read entry from directory handle

34. **rewinddir**
    ```php
    rewinddir($stream): void
    ```
    - Rewind directory handle

35. **scandir**
    ```php
    scandir($directory, $sorting_order = SCANDIR_SORT_ASCENDING, $context = null): array|false
    ```
    - List files and directories inside path

36. **mkdir**
    ```php
    mkdir($directory, $permissions = 0777, $recursive = false, $context = null): bool
    ```
    - Make directory

37. **rmdir**
    ```php
    rmdir($directory, $context = null): bool
    ```
    - Remove directory

38. **glob**
    ```php
    glob($pattern, $flags = 0): array|false
    ```
    - Find pathnames matching a pattern
    - Support flags: GLOB_MARK, GLOB_NOSORT, GLOB_NOCHECK, GLOB_NOESCAPE, GLOB_BRACE, GLOB_ONLYDIR, GLOB_ONLYFILE

39. **getcwd**
    ```php
    getcwd(): string|false
    ```
    - Gets current working directory

40. **chdir**
    ```php
    chdir($directory): bool
    ```
    - Change directory

### File Manipulation

41. **copy**
    ```php
    copy($source, $dest, $context = null): bool
    ```
    - Copies file

42. **rename**
    ```php
    rename($oldname, $newname, $context = null): bool
    ```
    - Renames a file or directory

43. **unlink**
    ```php
    unlink($filename, $context = null): bool
    ```
    - Deletes a file (already implemented)

44. **tempnam**
    ```php
    tempnam($directory, $prefix): string|false
    ```
    - Create file with unique file name

45. **tmpfile**
    ```php
    tmpfile(): resource|false
    ```
    - Create temporary file

46. **realpath**
    ```php
    realpath($path): string|false
    ```
    - Returns canonicalized absolute pathname

47. **symlink**
    ```php
    symlink($target, $link): bool
    ```
    - Creates symbolic link

48. **link**
    ```php
    link($target, $link): bool
    ```
    - Create hard link

49. **lchown**
    ```php
    lchown($filename, $user): bool
    ```
    - Changes user ownership of symlink

50. **lchgrp**
    ```php
    lchgrp($filename, $group): bool
    ```
    - Changes group ownership of symlink

51. **readlink**
    ```php
    readlink($path): string|false
    ```
    - Returns target of symbolic link

### File Permissions

52. **chmod**
    ```php
    chmod($filename, $permissions): bool
    ```
    - Changes file mode

53. **chown**
    ```php
    chown($filename, $user): bool
    ```
    - Changes file owner

54. **chgrp**
    ```php
    chgrp($filename, $group): bool
    ```
    - Changes file group

55. **is_executable**
    ```php
    is_executable($filename): bool
    ```
    - Tells whether filename is executable

56. **is_link**
    ```php
    is_link($filename): bool
    ```
    - Tells whether filename is a symbolic link

57. **is_uploaded_file**
    ```php
    is_uploaded_file($filename): bool
    ```
    - Tells whether file was uploaded via HTTP POST

58. **move_uploaded_file**
    ```php
    move_uploaded_file($from, $to): bool
    ```
    - Moves uploaded file to new location

### Path Operations

59. **dirname**
    ```php
    dirname($path, $levels = 1): string
    ```
    - Return parent directory path

60. **basename**
    ```php
    basename($path, $suffix = ""): string
    ```
    - Return trailing name component of path

61. **pathinfo**
    ```php
    pathinfo($path, $flags = PATHINFO_ALL): array|string
    ```
    - Returns information about file path
    - Support PATHINFO_DIRNAME, PATHINFO_BASENAME, PATHINFO_EXTENSION, PATHINFO_FILENAME

### File System State

62. **disk_free_space**
    ```php
    disk_free_space($directory): float|false
    ```
    - Returns available space on filesystem or disk partition

63. **disk_total_space**
    ```php
    disk_total_space($directory): float|false
    ```
    - Returns total size of filesystem or disk partition

## Implementation Plan

### Phase 1: File Streams (fopen, fclose, fread, fwrite)

**File:** `runtime/builtins/file_streams.rs` (new)

**Tasks:**
- [ ] Implement fopen with all modes
- [ ] Implement fclose
- [ ] Implement fread
- [ ] Implement fwrite
- [ ] Add tests

### Phase 2: File Navigation (fseek, ftell, rewind, feof)

**File:** `runtime/builtins/file_streams.rs` (extend)

**Tasks:**
- [ ] Implement fseek
- [ ] Implement ftell
- [ ] Implement rewind
- [ ] Implement feof
- [ ] Add tests

### Phase 3: Line/Character Operations

**File:** `runtime/builtins/file_streams.rs` (extend)

**Tasks:**
- [ ] Implement fgets
- [ ] Implement fgetc
- [ ] Implement fgetcsv
- [ ] Implement fputcsv
- [ ] Add tests

### Phase 4: File Information

**File:** `runtime/builtins/file_info.rs` (new)

**Tasks:**
- [ ] Implement file
- [ ] Implement fileatime
- [ ] Implement filectime
- [ ] Implement filegroup
- [ ] Implement fileinode
- [ ] Implement fileowner
- [ ] Implement fileperms
- [ ] Implement filetype
- [ ] Implement stat
- [ ] Implement lstat
- [ ] Implement fstat
- [ ] Implement touch
- [ ] Implement clearstatcache
- [ ] Add tests

### Phase 5: Directory Operations

**File:** `runtime/builtins/directory.rs` (new)

**Tasks:**
- [ ] Implement opendir
- [ ] Implement closedir
- [ ] Implement readdir
- [ ] Implement rewinddir
- [ ] Implement scandir
- [ ] Implement mkdir
- [ ] Implement rmdir
- [ ] Implement glob
- [ ] Implement getcwd
- [ ] Implement chdir
- [ ] Add tests

### Phase 6: File Manipulation

**File:** `runtime/builtins/file_ops.rs` (new)

**Tasks:**
- [ ] Implement copy
- [ ] Implement rename
- [ ] Implement tempnam
- [ ] Implement tmpfile
- [ ] Implement realpath
- [ ] Implement symlink
- [ ] Implement link
- [ ] Implement lchown
- [ ] Implement lchgrp
- [ ] Implement readlink
- [ ] Add tests

### Phase 7: File Permissions

**File:** `runtime/builtins/file_perms.rs` (new)

**Tasks:**
- [ ] Implement chmod
- [ ] Implement chown
- [ ] Implement chgrp
- [ ] Implement is_executable
- [ ] Implement is_link
- [ ] Implement is_uploaded_file
- [ ] Implement move_uploaded_file
- [ ] Add tests

### Phase 8: Path Operations

**File:** `runtime/builtins/path.rs` (new)

**Tasks:**
- [ ] Implement dirname
- [ ] Implement basename
- [ ] Implement pathinfo
- [ ] Add tests

### Phase 9: Additional File Stream Operations

**File:** `runtime/builtins/file_streams.rs` (extend)

**Tasks:**
- [ ] Implement fpassthru
- [ ] Implement fflush
- [ ] Implement flock
- [ ] Implement ftruncate
- [ ] Add tests

### Phase 10: File System State

**File:** `runtime/builtins/filesystem.rs` (new)

**Tasks:**
- [ ] Implement disk_free_space
- [ ] Implement disk_total_space
- [ ] Add tests

### Phase 11: Error Handling and Context

**Tasks:**
- [ ] Implement stream contexts (basic)
- [ ] Handle file system errors properly
- [ ] Match PHP error messages
- [ ] Add error handling tests

### Phase 12: Comprehensive Tests

**File:** `tests/filesystem/` (new directory)

Test coverage:
- File operations (open, read, write, close)
- File navigation (seek, tell, rewind)
- Line and character operations
- Directory operations
- Path operations
- File permissions
- File information
- CSV operations
- Symbolic links
- Error conditions
- Edge cases

## Implementation Details

### File Handle Representation

```rust
// In runtime/mod.rs or runtime/file.rs
pub struct FileHandle {
    file: std::fs::File,
    mode: FileMode,
    path: String,
}

pub enum FileMode {
    Read,
    Write,
    Append,
    ReadWrite,
    ReadAppend,
    Create,
    CreateRead,
}
```

### Glob Pattern Matching

```rust
use glob::glob;

fn php_glob(pattern: &str, flags: i64) -> Result<Vec<String>, String> {
    // Convert PHP glob pattern to glob crate pattern
    // Handle GLOB_* flags
    // Return matching paths
}
```

### CSV Parsing

```rust
fn parse_csv_line(line: &str, separator: char, enclosure: char, escape: char) -> Vec<String> {
    // Parse CSV fields with proper handling of quoted fields
    // Support escape characters
}
```

## Dependencies

- `glob = "0.3"` - Glob pattern matching

Optional for advanced features:
- `tempfile = "3.8"` - Temporary file handling
- `walkdir = "2.4"` - Directory traversal

## Testing Strategy

1. **Unit Tests**: Each function
2. **Integration Tests**: Combined operations
3. **Error Handling Tests**: Invalid operations, permissions
4. **Edge Cases**: Large files, special characters, symlinks
5. **Compatibility Tests**: Match PHP 8.x behavior

## Success Criteria

- All file stream operations work
- Directory operations work correctly
- Path operations handle all cases
- File information functions return correct data
- CSV parsing/encoding works
- All tests pass

## Performance Considerations

- Use buffered I/O where appropriate
- Minimize system calls
- Cache file information when possible (clearstatcache)
- Efficient glob pattern matching

## Security Considerations

- Path traversal attacks (validate paths)
- Directory traversal limitations
- Permission checks
- Safe file operations

## Open Questions

- Should we implement stream contexts and wrappers?
- How to handle file locking on different OS?
- Should we implement upload file handling separately?

## References

- PHP Filesystem documentation: https://www.php.net/manual/en/book.filesystem.php
- PHP Streams documentation: https://www.php.net/manual/en/book.stream.php

## Related Plans

- Stream wrappers and context (future)
- Upload handling (future)
