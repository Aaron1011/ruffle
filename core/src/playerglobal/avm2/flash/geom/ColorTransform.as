package flash.geom {
	public class ColorTransform {
		public var alphaMultiplier:Number = 1;
		public var alphaOffset:Number = 0;
		public var blueMultiplier:Number = 1;
		public var blueOffset:Number = 0;
		public var redMultiplier:Number = 1;
		public var redOffset:Number = 0;
		public var greenMultiplier:Number = 1;
		public var greenOffset:Number = 0;

		public function ColorTransform(redMultiplier:Number = 1.0, greenMultiplier:Number = 1.0, blueMultiplier:Number = 1.0, alphaMultiplier:Number = 1.0, redOffset:Number = 0, greenOffset:Number = 0, blueOffset:Number = 0, alphaOffset:Number = 0) {
		}

		public function set color(value:uint):void {
		}
	}
}
