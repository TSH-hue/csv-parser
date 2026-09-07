from pathlib import Path
import subprocess,json
ROOT=Path(__file__).resolve().parent
out=ROOT/'checkpoint-build';out.mkdir(exist_ok=True)
results=[]
for path in sorted((ROOT/'checkpoints').glob('*.rs')):
    binary=out/(path.stem+'-test.exe')
    subprocess.run(['rustc','--edition=2024','--test',str(path),'-o',str(binary)],check=True)
    test=subprocess.run([str(binary)],capture_output=True,text=True,check=True)
    binary=out/(path.stem+'-demo.exe')
    subprocess.run(['rustc','--edition=2024',str(path),'-o',str(binary)],check=True)
    run=subprocess.run([str(binary)],capture_output=True,text=True,encoding='utf-8',check=True)
    results.append(dict(file=str(path.relative_to(ROOT)),tests=test.stdout,output=run.stdout))
    print(path.name,'passed',flush=True)
(ROOT/'checkpoint-verification.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
