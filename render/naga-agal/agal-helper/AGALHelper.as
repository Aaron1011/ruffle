package {
	import flash.display3D.Context3DProgramType;
	import com.adobe.utils.AGALMiniAssembler;
	import flash.utils.ByteArray;

	public class AGALHelper {
		public function AGALHelper() {
			// Edit these with the AGAL you want to compile

			var vertexShader:AGALMiniAssembler = new AGALMiniAssembler();
			var vertexBytes = vertexShader.assemble(Context3DProgramType.VERTEX,
				"log vt0, va0\t\t\t\n" +
				"exp vt1, vt0           \n" +
				"pow vt2, vt1, va0      \n" +
				"sge vt3, vt2, va0\t    \n" +
				"m33 vt4, vc0, vt3      \n" +
				"m34 vt5, vc2, vt3      \n" +
				"add op,  vt4, vt5",
				2);
			trace("Vertex shader:");
			printArray(vertexBytes);

			var fragmentShader:AGALMiniAssembler = new AGALMiniAssembler();
			var fragmentBytes = vertexShader.assemble(Context3DProgramType.FRAGMENT,
				"ddx ft0, v0                \n" +
				"ddy ft1, ft0\t\t\t\t\n" +
				"kil ft1.x                  \n" +
				"mov oc, ft0   \t\t\t    \n",
				2);

			trace("Fragment shader:");
			printArray(fragmentBytes);
		}

		private function printArray(data:ByteArray) {
			var out = "&[";
			data.position = 0;
			while (data.bytesAvailable != 0) {
				out += data.readUnsignedByte();
				out += ",";
			}
			out += "]";
			trace(out);
		}
	}
}