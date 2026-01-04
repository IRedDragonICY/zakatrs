/// Main entry point for the Zakat library.
library;

// Export core API
export 'src/ffi/api/zakat.dart';
export 'src/ffi/api/assets.dart';
export 'src/ffi/api/types.dart';

// Export extensions
export 'src/extensions.dart';

// Export initialization
export 'src/ffi/frb_generated.dart' show RustLib;
