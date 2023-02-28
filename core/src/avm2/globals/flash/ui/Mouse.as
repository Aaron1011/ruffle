package flash.ui {
    public final class Mouse {
        public static native function hide(): void;
        public static native function show(): void;

         public static function get supportsCursor():Boolean {
            return true;
         }
    }
}
