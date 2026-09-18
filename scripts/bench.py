# 本機測速（需 ComfyUI 已在 8188 跑著）：~/ComfyUI/.venv/bin/python scripts/bench.py [zimage|flux|qwen|edit ...]   每個模型跑兩張，第二張才是穩定速度；同時回報 ComfyUI 程序的峰值 RSS（統一記憶體占用）。
import json, time, urllib.request, sys
API="http://127.0.0.1:8188"; W,H=1344,768
P="Photorealistic portrait of an adult woman in her late 20s, deep brown short hair, light freckles across the nose, a small mole at the tail of the left eyebrow, slightly asymmetric smile, natural window light, 35mm, sitting on a black leather sofa in a dark room, realistic skin texture"
def zimage(seed):
    return {"1":{"class_type":"UNETLoader","inputs":{"unet_name":"z_image_turbo_bf16.safetensors","weight_dtype":"default"}},
     "2":{"class_type":"CLIPLoader","inputs":{"clip_name":"qwen_3_4b.safetensors","type":"lumina2","device":"default"}},
     "3":{"class_type":"VAELoader","inputs":{"vae_name":"ae.safetensors"}},
     "4":{"class_type":"ModelSamplingAuraFlow","inputs":{"model":["1",0],"shift":3}},
     "5":{"class_type":"CLIPTextEncode","inputs":{"clip":["2",0],"text":P}},
     "6":{"class_type":"ConditioningZeroOut","inputs":{"conditioning":["5",0]}},
     "7":{"class_type":"EmptySD3LatentImage","inputs":{"width":W,"height":H,"batch_size":1}},
     "8":{"class_type":"KSampler","inputs":{"model":["4",0],"positive":["5",0],"negative":["6",0],"latent_image":["7",0],"seed":seed,"steps":8,"cfg":1,"sampler_name":"res_multistep","scheduler":"simple","denoise":1}},
     "9":{"class_type":"VAEDecode","inputs":{"samples":["8",0],"vae":["3",0]}},
     "10":{"class_type":"SaveImage","inputs":{"images":["9",0],"filename_prefix":"bench/zimage"}}}
def flux(seed):
    return {"1":{"class_type":"UnetLoaderGGUF","inputs":{"unet_name":"flux1-krea-dev-Q8_0.gguf"}},
     "2":{"class_type":"DualCLIPLoader","inputs":{"clip_name1":"clip_l.safetensors","clip_name2":"t5xxl_fp8_e4m3fn_scaled.safetensors","type":"flux","device":"default"}},
     "3":{"class_type":"VAELoader","inputs":{"vae_name":"ae.safetensors"}},
     "5":{"class_type":"CLIPTextEncode","inputs":{"clip":["2",0],"text":P}},
     "6":{"class_type":"ConditioningZeroOut","inputs":{"conditioning":["5",0]}},
     "7":{"class_type":"EmptySD3LatentImage","inputs":{"width":W,"height":H,"batch_size":1}},
     "8":{"class_type":"KSampler","inputs":{"model":["1",0],"positive":["5",0],"negative":["6",0],"latent_image":["7",0],"seed":seed,"steps":20,"cfg":1,"sampler_name":"euler","scheduler":"simple","denoise":1}},
     "9":{"class_type":"VAEDecode","inputs":{"samples":["8",0],"vae":["3",0]}},
     "10":{"class_type":"SaveImage","inputs":{"images":["9",0],"filename_prefix":"bench/flux_krea"}}}
def qwen(seed):
    return {"1":{"class_type":"UNETLoader","inputs":{"unet_name":"qwen_image_fp8_e4m3fn.safetensors","weight_dtype":"default"}},
     "1b":{"class_type":"LoraLoaderModelOnly","inputs":{"model":["1",0],"lora_name":"Qwen-Image-Lightning-8steps-V2.0.safetensors","strength_model":1}},
     "2":{"class_type":"CLIPLoader","inputs":{"clip_name":"qwen_2.5_vl_7b_fp8_scaled.safetensors","type":"qwen_image","device":"default"}},
     "3":{"class_type":"VAELoader","inputs":{"vae_name":"qwen_image_vae.safetensors"}},
     "4":{"class_type":"ModelSamplingAuraFlow","inputs":{"model":["1b",0],"shift":3.1}},
     "5":{"class_type":"CLIPTextEncode","inputs":{"clip":["2",0],"text":P}},
     "6":{"class_type":"CLIPTextEncode","inputs":{"clip":["2",0],"text":""}},
     "7":{"class_type":"EmptySD3LatentImage","inputs":{"width":W,"height":H,"batch_size":1}},
     "8":{"class_type":"KSampler","inputs":{"model":["4",0],"positive":["5",0],"negative":["6",0],"latent_image":["7",0],"seed":seed,"steps":8,"cfg":1,"sampler_name":"euler","scheduler":"simple","denoise":1}},
     "9":{"class_type":"VAEDecode","inputs":{"samples":["8",0],"vae":["3",0]}},
     "10":{"class_type":"SaveImage","inputs":{"images":["9",0],"filename_prefix":"bench/qwen_image"}}}
def edit(seed):
    ins="Same woman, same face, same hair and clothes, same black leather sofa and lighting. Turn her head and body 45 degrees to her left (three-quarter view), looking slightly off camera, natural relaxed expression."
    return {"1":{"class_type":"UNETLoader","inputs":{"unet_name":"qwen_image_edit_2511_int8_convrot.safetensors","weight_dtype":"default"}},
     "1b":{"class_type":"LoraLoaderModelOnly","inputs":{"model":["1",0],"lora_name":"Qwen-Image-Edit-2511-Lightning-4steps-V1.0-bf16.safetensors","strength_model":1}},
     "2":{"class_type":"CLIPLoader","inputs":{"clip_name":"qwen_2.5_vl_7b_fp8_scaled.safetensors","type":"qwen_image","device":"default"}},
     "3":{"class_type":"VAELoader","inputs":{"vae_name":"qwen_image_vae.safetensors"}},
     "4":{"class_type":"ModelSamplingAuraFlow","inputs":{"model":["1b",0],"shift":3.1}},
     "4b":{"class_type":"CFGNorm","inputs":{"model":["4",0],"strength":1}},
     "11":{"class_type":"LoadImage","inputs":{"image":"bench_portrait.png"}},
     "12":{"class_type":"FluxKontextImageScale","inputs":{"image":["11",0]}},
     "13":{"class_type":"VAEEncode","inputs":{"pixels":["12",0],"vae":["3",0]}},
     "5":{"class_type":"TextEncodeQwenImageEditPlus","inputs":{"clip":["2",0],"prompt":ins,"vae":["3",0],"image1":["12",0]}},
     "6":{"class_type":"TextEncodeQwenImageEditPlus","inputs":{"clip":["2",0],"prompt":"","vae":["3",0],"image1":["12",0]}},
     "8":{"class_type":"KSampler","inputs":{"model":["4b",0],"positive":["5",0],"negative":["6",0],"latent_image":["13",0],"seed":seed,"steps":4,"cfg":1,"sampler_name":"euler","scheduler":"simple","denoise":1}},
     "9":{"class_type":"VAEDecode","inputs":{"samples":["8",0],"vae":["3",0]}},
     "10":{"class_type":"SaveImage","inputs":{"images":["9",0],"filename_prefix":"bench/qwen_edit"}}}
import threading
def mem_used_gb():
    try:
        d=json.load(urllib.request.urlopen(API+"/system_stats"))["devices"][0]; return (d["vram_total"]-d["vram_free"])/1e9
    except Exception: return 0.0
class Peak:  # 取樣 torch MPS driver-allocated memory（統一記憶體實際占用）
    def __init__(s): s.peak=0.0; s.stop=False; threading.Thread(target=s._loop,daemon=True).start()
    def _loop(s):
        while not s.stop: s.peak=max(s.peak,mem_used_gb()); time.sleep(0.5)
def run(g):
    pk=Peak()
    req=urllib.request.Request(API+"/prompt",data=json.dumps({"prompt":g}).encode(),headers={"Content-Type":"application/json"})
    r=json.load(urllib.request.urlopen(req))
    if "prompt_id" not in r: sys.exit(json.dumps(r,indent=1,ensure_ascii=False)[:3000])
    pid=r["prompt_id"]; t=time.time()
    while True:
        h=json.load(urllib.request.urlopen(API+"/history/"+pid))
        if pid in h and h[pid]["status"]["status_str"]=="error": sys.exit(json.dumps(h[pid]["status"],indent=1)[:3000])
        if pid in h and h[pid]["status"]["completed"]:
            pk.stop=True; return time.time()-t, h[pid]["outputs"]["10"]["images"][0]["filename"], pk.peak
        time.sleep(1)
which=sys.argv[1:] or ["zimage","flux","qwen","edit"]
for name in which:
    for i,seed in enumerate((1234,5678),1):
        s,f,m=run(globals()[name](seed)); print(f"{name:7s} run {i}: {s:6.1f}s  peak mem {m:5.1f} GB  -> output/bench/{f}", flush=True)
