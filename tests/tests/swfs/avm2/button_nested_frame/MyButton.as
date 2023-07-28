package  {
	
	import flash.display.SimpleButton;
	
	
	public class MyButton extends SimpleButton {
		
		
		public function MyButton() {
			trace("Calling MyButton super(): this.parent[\"myButton\"] = " + this.parent["myButton"]);
			super();
			trace("Called MyButton super(): this.parent[\"myButton\"] = " + this.parent["myButton"]);
		}
	}
	
}
