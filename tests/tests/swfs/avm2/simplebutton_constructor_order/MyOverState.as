package  {
	
	import flash.display.MovieClip;
	
	
	public class MyOverState extends MovieClip {
		
		
		public function MyOverState() {
			trace("MyOverState constructor called");
			new EventWatcher(this);
		}
	}
	
}
